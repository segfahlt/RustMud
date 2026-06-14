use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Deserialize;

use super::fixture::Fixture;
use super::object::{ObjectInstance, ObjectTemplate};
use super::room::Direction;
use super::{Room, RoomRef, World, Zone};

// --- File format structs ---
// Public so the schema binary can generate JSON Schema from them.
// The rest of the game still works through World/Zone/Room — never these directly.

/// Minimal object reference in a zone file: spawns one instance of the named template.
#[derive(Deserialize, JsonSchema)]
pub struct ObjectSpawnFile {
    pub template_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct RoomFile {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub exits: HashMap<String, RoomRef>,
    #[serde(default)]
    pub fixtures: Vec<Fixture>,
    #[serde(default)]
    pub objects: Vec<ObjectSpawnFile>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ZoneFile {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub rooms: Vec<RoomFile>,
    /// Templates defined here are available to all rooms in this zone.
    #[serde(default)]
    pub object_templates: Vec<ObjectTemplate>,
}

// --- LoadError ---

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Json { path: PathBuf, source: serde_json::Error },
    UnknownDirection { path: PathBuf, room_id: u32, direction: String },
    UnknownTemplate { path: PathBuf, room_id: u32, template_id: String },
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
            LoadError::UnknownTemplate { path, room_id, template_id } => {
                write!(f, "In {}: room {}: unknown template '{}'",
                    path.display(), room_id, template_id)
            }
            LoadError::InvalidWorld(errors) => {
                write!(f, "World validation failed:\n  {}", errors.join("\n  "))
            }
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

// --- Public API ---

pub fn load_world(data_dir: &Path) -> Result<World, LoadError> {
    let zones_dir = data_dir.join("zones");

    let mut paths: Vec<PathBuf> = fs::read_dir(&zones_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();
    paths.sort();

    let mut world = World::new();
    for path in paths {
        load_zone_into(&path, &mut world)?;
    }

    let errors = world.validate();
    if !errors.is_empty() {
        return Err(LoadError::InvalidWorld(errors));
    }

    Ok(world)
}

// --- Private helpers ---

fn load_zone_into(path: &Path, world: &mut World) -> Result<(), LoadError> {
    let content = fs::read_to_string(path)?;
    let zone_file: ZoneFile = serde_json::from_str(&content)
        .map_err(|e| LoadError::Json { path: path.to_path_buf(), source: e })?;

    // Register this zone's templates in the global registry.
    for tmpl in zone_file.object_templates {
        world.object_registry.insert(tmpl.id.clone(), tmpl);
    }

    let mut zone = Zone::new(zone_file.id, zone_file.name, zone_file.description);

    for room_file in zone_file.rooms {
        let room_id = room_file.id;
        let mut exits = HashMap::new();

        for (dir_str, room_ref) in room_file.exits {
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

        // Spawn object instances from template references.
        let mut objects = Vec::new();
        for spawn in room_file.objects {
            if !world.object_registry.contains_key(&spawn.template_id) {
                return Err(LoadError::UnknownTemplate {
                    path: path.to_path_buf(),
                    room_id,
                    template_id: spawn.template_id,
                });
            }
            objects.push(ObjectInstance::new(spawn.template_id));
        }

        zone.add_room(Room {
            id: room_id,
            name: room_file.name,
            description: room_file.description,
            exits,
            fixtures: room_file.fixtures,
            objects,
        });
    }

    world.add_zone(zone);
    Ok(())
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
        assert!(zf.object_templates.is_empty());
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
        assert!(world.get_room(1, 1).is_some(), "Cryo-Bay (zone 1, room 1) should exist");
        assert!(world.get_room(2, 2).is_some(), "Intake Lobby (zone 2, room 2) should exist");
        assert!(world.validate().is_empty(), "loaded world should be valid");
    }
}
