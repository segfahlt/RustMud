use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

pub mod area;
pub mod hex;
pub mod loader;
pub mod object;
pub mod room;
pub mod worldmap;
pub mod zone;

pub use area::Area;
pub use hex::{
    AreaRef, EvolutionStage, ExitDestination, FixtureRef, HexCoord, PlayerLocation,
};
pub use object::{FixturePermanence, ObjectInstance, ObjectRegistry, ObjectTemplate};
pub use room::{Direction, Room};
pub use worldmap::WorldMap;
pub use zone::Zone;

pub struct World {
    zones:           HashMap<HexCoord, Zone>,
    pub rooms:       HashMap<u32, Room>,
    pub object_registry: ObjectRegistry,
    pub world_map:       WorldMap,
    room_id_seq:     AtomicU32,
}

impl Default for World {
    fn default() -> Self { Self::new() }
}

impl World {
    pub fn new() -> Self {
        World {
            zones:           HashMap::new(),
            rooms:           HashMap::new(),
            object_registry: HashMap::new(),
            world_map:       WorldMap::empty(),
            room_id_seq:     AtomicU32::new(0),
        }
    }

    /// Returns the next unique room ID (increments the sequence).
    pub fn next_room_id(&self) -> u32 {
        self.room_id_seq.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Seeds the sequence from the highest previously issued ID.
    /// Called by the loader after all static rooms are registered.
    pub fn seed_room_id_seq(&self, last_issued: u32) {
        self.room_id_seq.store(last_issued, Ordering::Relaxed);
    }

    /// Returns the last issued room ID (the value to persist on shutdown).
    pub fn room_id_seq_snapshot(&self) -> u32 {
        self.room_id_seq.load(Ordering::Relaxed)
    }

    // --- Zone / Area API ---

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.coord, zone);
    }

    pub fn get_zone(&self, coord: HexCoord) -> Option<&Zone> {
        self.zones.get(&coord)
    }

    pub fn get_area(&self, area_ref: AreaRef) -> Option<&Area> {
        self.zones.get(&area_ref.zone)?.get_area(area_ref.area_id)
    }

    pub fn get_area_mut(&mut self, area_ref: AreaRef) -> Option<&mut Area> {
        self.zones.get_mut(&area_ref.zone)?.get_area_mut(area_ref.area_id)
    }

    pub fn zone_coords(&self) -> Vec<HexCoord> {
        let mut coords: Vec<HexCoord> = self.zones.keys().copied().collect();
        coords.sort_by_key(|c| (c.q, c.r));
        coords
    }

    pub fn get_zone_name(&self, coord: HexCoord) -> Option<&str> {
        self.zones.get(&coord).map(|z| z.name.as_str())
    }

    // --- Room API ---

    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id, room);
    }

    pub fn get_room(&self, room_id: u32) -> Option<&Room> {
        self.rooms.get(&room_id)
    }

    pub fn get_room_mut(&mut self, room_id: u32) -> Option<&mut Room> {
        self.rooms.get_mut(&room_id)
    }

    // --- Validation ---

    /// Finds a gateway object in the given area that connects to a room via `dir`.
    /// A gateway object has `connects_to_room` set on its template.
    /// If its `direction` field is set, it only triggers for that direction.
    pub fn find_gateway(&self, area_ref: AreaRef, dir: Direction) -> Option<u32> {
        let area = self.get_area(area_ref)?;
        for obj in &area.objects {
            let tmpl = self.object_registry.get(&obj.template_id)?;
            let room_id = tmpl.connects_to_room?;
            let fixture_dir = tmpl.direction.as_deref()
                .and_then(|d| d.parse::<Direction>().ok());
            if fixture_dir.is_none() || fixture_dir == Some(dir) {
                return Some(room_id);
            }
        }
        None
    }

    // Checks that all Area exits point to existing Areas,
    // and all Room exits point to existing Rooms or valid FixtureRefs.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for coord in self.zone_coords() {
            let zone = self.get_zone(coord).unwrap();
            for area_id in zone.area_ids() {
                let area_ref = AreaRef { zone: coord, area_id };
                let area = self.get_area(area_ref).unwrap();
                for (dir, dest) in &area.exits {
                    if self.get_area(*dest).is_none() {
                        errors.push(format!(
                            "Area ({},{}) id={} exit {:?}: points to missing area zone=({},{}) id={}",
                            coord.q, coord.r, area_id, dir,
                            dest.zone.q, dest.zone.r, dest.area_id
                        ));
                    }
                }
            }
        }

        for room_id in self.rooms.keys() {
            let room = self.get_room(*room_id).unwrap();
            for (dir, dest) in &room.exits {
                match dest {
                    ExitDestination::Room { room_id: target_id } => {
                        if self.get_room(*target_id).is_none() {
                            errors.push(format!(
                                "Room {} exit {:?}: points to missing room {}",
                                room_id, dir, target_id
                            ));
                        }
                    }
                    ExitDestination::Fixture(fixture_ref) => {
                        let area_ref = AreaRef { zone: fixture_ref.zone, area_id: fixture_ref.area_id };
                        if let Some(area) = self.get_area(area_ref) {
                            if !area.objects.iter().any(|o| o.template_id == fixture_ref.fixture_id) {
                                errors.push(format!(
                                    "Room {} exit {:?}: fixture '{}' not found in area ({},{}) id={}",
                                    room_id, dir, fixture_ref.fixture_id,
                                    fixture_ref.zone.q, fixture_ref.zone.r, fixture_ref.area_id
                                ));
                            }
                        } else {
                            errors.push(format!(
                                "Room {} exit {:?}: fixture ref points to missing area ({},{}) id={}",
                                room_id, dir,
                                fixture_ref.zone.q, fixture_ref.zone.r, fixture_ref.area_id
                            ));
                        }
                    }
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn coord(q: i32, r: i32) -> HexCoord { HexCoord::new(q, r) }

    fn make_world() -> World {
        let mut world = World::new();
        let mut zone = Zone::new(coord(0, 0), "Test Zone", "A test zone.");
        zone.add_area(Area {
            id: 1,
            name: "Area One".to_string(),
            description: "First area.".to_string(),
            exits: HashMap::from([
                (Direction::North, AreaRef { zone: coord(0, 0), area_id: 2 }),
            ]),
            ..Area::default()
        });
        zone.add_area(Area {
            id: 2,
            name: "Area Two".to_string(),
            description: "Second area.".to_string(),
            exits: HashMap::from([
                (Direction::South, AreaRef { zone: coord(0, 0), area_id: 1 }),
            ]),
            ..Area::default()
        });
        world.add_zone(zone);
        world
    }

    #[test]
    fn get_existing_area() {
        let w = make_world();
        assert!(w.get_area(AreaRef { zone: coord(0, 0), area_id: 1 }).is_some());
    }

    #[test]
    fn get_missing_area_returns_none() {
        let w = make_world();
        assert!(w.get_area(AreaRef { zone: coord(0, 0), area_id: 99 }).is_none());
    }

    #[test]
    fn get_missing_zone_returns_none() {
        let w = make_world();
        assert!(w.get_area(AreaRef { zone: coord(99, 99), area_id: 1 }).is_none());
    }

    #[test]
    fn zone_coords_are_sorted() {
        let mut world = World::new();
        world.add_zone(Zone::new(coord(3, 0), "C", ""));
        world.add_zone(Zone::new(coord(1, 0), "A", ""));
        world.add_zone(Zone::new(coord(2, 0), "B", ""));
        let coords = world.zone_coords();
        assert_eq!(coords[0], coord(1, 0));
        assert_eq!(coords[1], coord(2, 0));
        assert_eq!(coords[2], coord(3, 0));
    }

    #[test]
    fn validate_clean_world_returns_no_errors() {
        assert!(make_world().validate().is_empty());
    }

    #[test]
    fn next_room_id_starts_above_zero() {
        let world = World::new();
        let id = world.next_room_id();
        assert!(id >= 1, "first id should be at least 1, got {id}");
    }

    #[test]
    fn next_room_id_increments() {
        let world = World::new();
        let a = world.next_room_id();
        let b = world.next_room_id();
        assert_eq!(b, a + 1);
    }

    #[test]
    fn seed_room_id_seq_respected() {
        let world = World::new();
        world.seed_room_id_seq(100);
        assert_eq!(world.next_room_id(), 101);
    }

    #[test]
    fn validate_detects_dead_area_exit() {
        let mut world = World::new();
        let mut zone = Zone::new(coord(0, 0), "Zone", "");
        zone.add_area(Area {
            id: 1,
            name: "Area".to_string(),
            exits: HashMap::from([
                (Direction::North, AreaRef { zone: coord(0, 0), area_id: 999 }),
            ]),
            ..Area::default()
        });
        world.add_zone(zone);
        assert!(!world.validate().is_empty());
    }
}
