use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Deserialize;

use super::area::Area;
use super::hex::{AreaRef, EvolutionStage, ExitDestination, HexCoord};
use super::object::{
    Bulk, FixturePermanence, Material, ObjectInstance, ObjectTemplate, Weight,
};
use super::room::{Direction, Room};
use super::{World, WorldMap, Zone};

// --- File format structs ---
// Public so the schema binary can generate JSON Schema from them.

/// Minimal object reference in a zone file: spawns one instance of the named template.
#[derive(Deserialize, JsonSchema)]
pub struct ObjectSpawnFile {
    pub template_id: String,
}

/// Inline fixture definition. The loader auto-registers an ObjectTemplate for each
/// fixture and spawns one ObjectInstance into the containing room or area.
#[derive(Deserialize, JsonSchema)]
pub struct FixtureFile {
    pub id:       String,
    pub names:    Vec<String>,
    pub category: super::object::ObjectCategory,
    #[serde(default)]
    pub state_lines: Option<HashMap<String, String>>,
    /// Shown on `examine <fixture>`. Becomes ObjectTemplate.description.
    pub examine:  String,
    #[serde(default)]
    pub read:     Option<String>,
    #[serde(default)]
    pub permanence: FixturePermanence,
    #[serde(default)]
    pub minimum_stage: Option<EvolutionStage>,
    #[serde(default)]
    pub connects_to_room: Option<u32>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub coherence_driven: bool,
    #[serde(default)]
    pub persist_state: bool,
}

impl FixtureFile {
    fn into_template(self) -> ObjectTemplate {
        let short = self.names.first().cloned().unwrap_or_default();
        ObjectTemplate {
            id:          self.id,
            names:       self.names,
            short,
            room_look:   String::new(),
            description: self.examine,
            read:        self.read,
            category:    self.category,
            weight:      Weight::default(),
            bulk:        Bulk::default(),
            material:    Material::default(),
            flags:       vec![],
            value:       0,
            equip_slot:      None,
            health_restore:  0,
            consume_message: None,
            state_lines:      self.state_lines,
            permanence:       Some(self.permanence),
            minimum_stage:    self.minimum_stage,
            connects_to_room: self.connects_to_room,
            direction:        self.direction,
            coherence_driven: self.coherence_driven,
            persist_state:    self.persist_state,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct AreaFile {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub exits: HashMap<String, AreaRef>,
    #[serde(default)]
    pub fixtures: Vec<FixtureFile>,
    #[serde(default)]
    pub objects: Vec<ObjectSpawnFile>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ZoneFile {
    pub q:            i32,
    pub r:            i32,
    pub name:         String,
    pub description:  String,
    #[serde(default)]
    pub biome_origin: String,
    #[serde(default = "default_coherence")]
    pub coherence:    u8,
    #[serde(default = "default_radius")]
    pub radius_steps: u8,
    pub areas:        Vec<AreaFile>,
    #[serde(default)]
    pub object_templates: Vec<ObjectTemplate>,
}

fn default_coherence() -> u8 { 50 }
fn default_radius() -> u8 { 1 }

// --- Building file format (data/buildings/*.json) ---

#[derive(Deserialize, JsonSchema)]
pub struct RoomFile {
    pub id:          u32,
    pub name:        String,
    pub description: String,
    #[serde(default)]
    pub breadcrumb_zone:     String,
    #[serde(default)]
    pub breadcrumb_building: String,
    #[serde(default)]
    pub exits:    HashMap<String, ExitDestination>,
    #[serde(default)]
    pub fixtures: Vec<FixtureFile>,
    #[serde(default)]
    pub objects:  Vec<ObjectSpawnFile>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BuildingFile {
    pub id:                  String,
    pub name:                String,
    pub breadcrumb_zone:     String,
    pub breadcrumb_building: String,
    pub rooms:               Vec<RoomFile>,
    #[serde(default)]
    pub object_templates:    Vec<ObjectTemplate>,
}

// --- LoadError ---

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Json { path: PathBuf, source: serde_json::Error },
    UnknownDirection { path: PathBuf, area_id: u32, direction: String },
    UnknownRoomDirection { path: PathBuf, room_id: u32, direction: String },
    UnknownTemplate { path: PathBuf, area_id: u32, template_id: String },
    DuplicateRoom { path: PathBuf, room_id: u32 },
    InvalidWorld(Vec<String>),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Json { path, source } => {
                write!(f, "JSON error in {}: {}", path.display(), source)
            }
            LoadError::UnknownDirection { path, area_id, direction } => {
                write!(f, "In {}: area {}: unknown direction '{}'",
                    path.display(), area_id, direction)
            }
            LoadError::UnknownRoomDirection { path, room_id, direction } => {
                write!(f, "In {}: room {}: unknown direction '{}'",
                    path.display(), room_id, direction)
            }
            LoadError::UnknownTemplate { path, area_id, template_id } => {
                write!(f, "In {}: area {}: unknown template '{}'",
                    path.display(), area_id, template_id)
            }
            LoadError::DuplicateRoom { path, room_id } => {
                write!(f, "In {}: room id {} already registered", path.display(), room_id)
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
    let mut world = World::new();

    let zones_dir = data_dir.join("zones");
    let mut zone_paths: Vec<PathBuf> = fs::read_dir(&zones_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
        .collect();
    zone_paths.sort();
    for path in zone_paths {
        load_zone_into(&path, &mut world)?;
    }

    let buildings_dir = data_dir.join("buildings");
    if buildings_dir.exists() {
        let mut building_paths: Vec<PathBuf> = fs::read_dir(&buildings_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
            .collect();
        building_paths.sort();
        for path in building_paths {
            load_building_into(&path, &mut world)?;
        }
    }

    world.world_map = load_worldmap(data_dir);

    let errors = world.validate();
    if !errors.is_empty() {
        return Err(LoadError::InvalidWorld(errors));
    }

    // Seed the room ID sequence above the highest statically assigned room ID.
    let max_static = world.rooms.keys().copied().max().unwrap_or(0);
    let file_val: u32 = fs::read_to_string(data_dir.join("state").join("last_room_id"))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    world.seed_room_id_seq(max_static.max(file_val));

    Ok(world)
}

/// Writes the current room ID sequence counter to `data/state/last_room_id`.
/// Call on every graceful shutdown so dynamic IDs remain unique across restarts.
pub fn flush_room_id_sequence(data_dir: &Path, world: &World) -> io::Result<()> {
    let path = data_dir.join("state").join("last_room_id");
    fs::write(path, format!("{}\n", world.room_id_seq_snapshot()))
}

pub fn load_worldmap(data_dir: &Path) -> WorldMap {
    let path = data_dir.join("world").join("map.txt");
    match fs::read_to_string(&path) {
        Ok(content) => WorldMap::from_rows(content.lines().map(String::from).collect()),
        Err(_)      => WorldMap::empty(),
    }
}

// --- Private helpers ---

fn load_zone_into(path: &Path, world: &mut World) -> Result<(), LoadError> {
    let content = fs::read_to_string(path)?;
    let zone_file: ZoneFile = serde_json::from_str(&content)
        .map_err(|e| LoadError::Json { path: path.to_path_buf(), source: e })?;

    for tmpl in zone_file.object_templates {
        world.object_registry.insert(tmpl.id.clone(), tmpl);
    }

    let coord = HexCoord::new(zone_file.q, zone_file.r);
    let mut zone = Zone::new(coord, zone_file.name, zone_file.description);
    zone.biome_origin = zone_file.biome_origin;
    zone.coherence    = zone_file.coherence;
    zone.radius_steps = zone_file.radius_steps;

    for area_file in zone_file.areas {
        let area_id = area_file.id;
        let mut exits = HashMap::new();

        for (dir_str, area_ref) in area_file.exits {
            let dir = dir_str.parse::<Direction>().map_err(|_| LoadError::UnknownDirection {
                path: path.to_path_buf(),
                area_id,
                direction: dir_str,
            })?;
            exits.insert(dir, area_ref);
        }

        let mut objects = Vec::new();

        for fixture in area_file.fixtures {
            let template_id = fixture.id.clone();
            let tmpl = fixture.into_template();
            world.object_registry.insert(template_id.clone(), tmpl);
            objects.push(ObjectInstance::new(template_id));
        }

        for spawn in area_file.objects {
            if !world.object_registry.contains_key(&spawn.template_id) {
                return Err(LoadError::UnknownTemplate {
                    path: path.to_path_buf(),
                    area_id,
                    template_id: spawn.template_id,
                });
            }
            objects.push(ObjectInstance::new(spawn.template_id));
        }

        zone.add_area(Area {
            id: area_id,
            name: area_file.name,
            description: area_file.description,
            exits,
            objects,
            ..Area::default()
        });
    }

    world.add_zone(zone);
    Ok(())
}

fn load_building_into(path: &Path, world: &mut World) -> Result<(), LoadError> {
    let content = fs::read_to_string(path)?;
    let building: BuildingFile = serde_json::from_str(&content)
        .map_err(|e| LoadError::Json { path: path.to_path_buf(), source: e })?;

    for tmpl in building.object_templates {
        world.object_registry.insert(tmpl.id.clone(), tmpl);
    }

    for room_file in building.rooms {
        let room_id = room_file.id;
        if world.rooms.contains_key(&room_id) {
            return Err(LoadError::DuplicateRoom { path: path.to_path_buf(), room_id });
        }

        let mut exits = HashMap::new();
        for (dir_str, dest) in room_file.exits {
            let dir = dir_str.parse::<Direction>().map_err(|_| LoadError::UnknownRoomDirection {
                path: path.to_path_buf(),
                room_id,
                direction: dir_str,
            })?;
            exits.insert(dir, dest);
        }

        let mut objects = Vec::new();

        for fixture in room_file.fixtures {
            let template_id = fixture.id.clone();
            let tmpl = fixture.into_template();
            world.object_registry.insert(template_id.clone(), tmpl);
            objects.push(ObjectInstance::new(template_id));
        }

        for spawn in room_file.objects {
            if !world.object_registry.contains_key(&spawn.template_id) {
                return Err(LoadError::UnknownTemplate {
                    path: path.to_path_buf(),
                    area_id: room_id,
                    template_id: spawn.template_id,
                });
            }
            objects.push(ObjectInstance::new(spawn.template_id));
        }

        let zone_label = if room_file.breadcrumb_zone.is_empty() {
            building.breadcrumb_zone.clone()
        } else {
            room_file.breadcrumb_zone
        };
        let bldg_label = if room_file.breadcrumb_building.is_empty() {
            building.breadcrumb_building.clone()
        } else {
            room_file.breadcrumb_building
        };

        world.add_room(Room {
            id: room_id,
            name: room_file.name,
            description: room_file.description,
            breadcrumb_zone: zone_label,
            breadcrumb_building: bldg_label,
            exits,
            objects,
        });
    }

    Ok(())
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn deserialize_minimal_zone() {
        let json = r#"{ "q": 0, "r": 0, "name": "Z", "description": "D", "areas": [] }"#;
        let zf: ZoneFile = serde_json::from_str(json).unwrap();
        assert_eq!(zf.q, 0);
        assert_eq!(zf.r, 0);
        assert_eq!(zf.name, "Z");
        assert!(zf.areas.is_empty());
        assert!(zf.object_templates.is_empty());
    }

    #[test]
    fn deserialize_area_with_exits() {
        let json = r#"{
            "id": 1, "name": "R", "description": "D",
            "exits": { "north": { "zone": { "q": 0, "r": 0 }, "area_id": 2 } }
        }"#;
        let af: AreaFile = serde_json::from_str(json).unwrap();
        let dest = af.exits.get("north").unwrap();
        assert_eq!(dest.zone.q, 0);
        assert_eq!(dest.zone.r, 0);
        assert_eq!(dest.area_id, 2);
    }

    #[test]
    fn deserialize_area_missing_exits_defaults_to_empty() {
        let json = r#"{ "id": 1, "name": "Dead End", "description": "." }"#;
        let af: AreaFile = serde_json::from_str(json).unwrap();
        assert!(af.exits.is_empty());
    }

    #[test]
    fn fixture_converts_to_template_and_spawns_instance() {
        let json = r#"{
            "id": "test_gate",
            "names": ["gate", "test gate"],
            "category": "structural",
            "state_lines": { "default": "A gate stands here." },
            "examine": "A solid gate.",
            "permanence": "permanent"
        }"#;
        let ff: FixtureFile = serde_json::from_str(json).unwrap();
        let tmpl = ff.into_template();
        assert_eq!(tmpl.id, "test_gate");
        assert!(tmpl.category.is_fixture());
        assert!(tmpl.state_lines.is_some());
        assert_eq!(tmpl.description, "A solid gate.");
    }

    #[test]
    fn load_world_succeeds_and_passes_validation() {
        let world = load_world(Path::new("data")).expect("world should load from data/");
        assert!(world.get_room(1).is_some(),  "Cryo-Bay (room 1) should exist");
        assert!(world.get_room(5).is_some(),  "Harbor Dock (room 5) should exist");
        assert!(world.get_room(10).is_some(), "South Gate — Interior (room 10) should exist");
        assert!(world.get_room(20).is_some(), "North Gate (room 20) should exist");
        assert!(world.validate().is_empty(), "loaded world should be valid");
    }
}
