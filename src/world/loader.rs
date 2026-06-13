use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Deserialize;

use super::room::Direction;
use super::{Room, RoomRef, World, Zone};

// --- File format structs ---
// Public so the schema binary can generate JSON Schema from them.
// The rest of the game still works through World/Zone/Room — never these directly.

#[derive(Deserialize, JsonSchema)]
pub struct RoomFile {
    pub id: u32,
    pub name: String,
    pub description: String,
    // `default` means a missing "exits" key deserializes as an empty map.
    // schemars reflects this: the field will not appear in "required".
    #[serde(default)]
    pub exits: HashMap<String, RoomRef>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ZoneFile {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub rooms: Vec<RoomFile>,
}

// --- LoadError ---

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Json { path: PathBuf, source: serde_json::Error },
    UnknownDirection { path: PathBuf, room_id: u32, direction: String },
    InvalidWorld(Vec<String>),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Json { path, source } => {
                write!(f, "JSON error in {}: {}", path.display(), source)
            }
            LoadError::UnknownDirection { path, room_id, direction } => {
                write!(f, "In {}: room {}: unknown direction '{}'",
                    path.display(), room_id, direction)
            }
            LoadError::InvalidWorld(errors) => {
                write!(f, "World validation failed:\n  {}", errors.join("\n  "))
            }
        }
    }
}

// `From` implementations let the `?` operator automatically convert
// io::Error into LoadError when used in functions returning Result<_, LoadError>.
impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

// --- Public API ---

pub fn load_world(data_dir: &Path) -> Result<World, LoadError> {
    let zones_dir = data_dir.join("zones");

    // Collect paths, filter to .json only, sort for deterministic load order.
    let mut paths: Vec<PathBuf> = fs::read_dir(&zones_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();
    paths.sort();

    let mut world = World::new();
    for path in paths {
        world.add_zone(load_zone(&path)?);
    }

    // Validate after all zones are loaded so cross-zone exits can be checked.
    let errors = world.validate();
    if !errors.is_empty() {
        return Err(LoadError::InvalidWorld(errors));
    }

    Ok(world)
}

// --- Private helpers ---

fn load_zone(path: &Path) -> Result<Zone, LoadError> {
    let content = fs::read_to_string(path)?;

    // serde_json::Error doesn't carry the file path, so we attach it manually.
    let zone_file: ZoneFile = serde_json::from_str(&content)
        .map_err(|e| LoadError::Json { path: path.to_path_buf(), source: e })?;

    let mut zone = Zone::new(zone_file.id, zone_file.name, zone_file.description);

    for room_file in zone_file.rooms {
        let room_id = room_file.id;
        let mut exits = HashMap::new();

        for (dir_str, room_ref) in room_file.exits {
            // parse::<Direction>() uses the FromStr impl on Direction.
            let dir = match dir_str.parse::<Direction>() {
                Ok(d) => d,
                Err(_) => return Err(LoadError::UnknownDirection {
                    path: path.to_path_buf(),
                    room_id,
                    direction: dir_str,
                }),
            };
            exits.insert(dir, room_ref);
        }

        zone.add_room(Room {
            id: room_id,
            name: room_file.name,
            description: room_file.description,
            exits,
        });
    }

    Ok(zone)
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn deserialize_minimal_zone() {
        let json = r#"{ "id": 1, "name": "Z", "description": "D", "rooms": [] }"#;
        let zf: ZoneFile = serde_json::from_str(json).unwrap();
        assert_eq!(zf.id, 1);
        assert_eq!(zf.name, "Z");
        assert!(zf.rooms.is_empty());
    }

    #[test]
    fn deserialize_room_with_exits() {
        let json = r#"{
            "id": 1, "name": "R", "description": "D",
            "exits": { "north": { "zone_id": 2, "room_id": 5 } }
        }"#;
        let rf: RoomFile = serde_json::from_str(json).unwrap();
        let dest = rf.exits.get("north").unwrap();
        assert_eq!(dest.zone_id, 2);
        assert_eq!(dest.room_id, 5);
    }

    #[test]
    fn deserialize_room_missing_exits_defaults_to_empty() {
        let json = r#"{ "id": 1, "name": "Dead End", "description": "." }"#;
        let rf: RoomFile = serde_json::from_str(json).unwrap();
        assert!(rf.exits.is_empty());
    }

    #[test]
    fn load_world_succeeds_and_passes_validation() {
        let world = load_world(Path::new("data")).expect("world should load from data/");
        assert!(world.get_room(1, 1).is_some(), "Town Square should exist");
        assert!(world.get_room(2, 2).is_some(), "Dark Clearing should exist");
        assert!(world.validate().is_empty(), "loaded world should be valid");
    }
}
