use std::collections::HashMap;

pub mod fixture;
pub mod loader;
pub mod object;
pub mod room;
pub mod zone;

pub use fixture::Fixture;
pub use object::{ObjectInstance, ObjectRegistry, ObjectTemplate};
pub use room::{Direction, Room, RoomRef};
pub use zone::Zone;

pub struct World {
    zones: HashMap<u32, Zone>,
    pub object_registry: ObjectRegistry,
}

impl World {
    pub fn new() -> Self {
        World {
            zones:           HashMap::new(),
            object_registry: HashMap::new(),
        }
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.id, zone);
    }

    pub fn get_zone(&self, zone_id: u32) -> Option<&Zone> {
        self.zones.get(&zone_id)
    }

    pub fn get_room(&self, zone_id: u32, room_id: u32) -> Option<&Room> {
        self.zones.get(&zone_id)?.get_room(room_id)
    }

    pub fn get_room_mut(&mut self, zone_id: u32, room_id: u32) -> Option<&mut Room> {
        self.zones.get_mut(&zone_id)?.get_room_mut(room_id)
    }

    // Returns zone IDs in sorted order.
    pub fn zone_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.zones.keys().copied().collect();
        ids.sort();
        ids
    }

    // Checks that every exit RoomRef points to a room that actually exists.
    // Returns a list of error strings — empty means the world is consistent.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for zone_id in self.zone_ids() {
            let zone = self.get_zone(zone_id).unwrap();
            for room_id in zone.room_ids() {
                let room = self.get_room(zone_id, room_id).unwrap();
                for (dir, dest) in &room.exits {
                    if self.get_room(dest.zone_id, dest.room_id).is_none() {
                        errors.push(format!(
                            "Zone {} Room {} exit {:?}: points to missing zone={} room={}",
                            zone_id, room_id, dir, dest.zone_id, dest.room_id
                        ));
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

    fn make_world() -> World {
        let mut world = World::new();
        let mut zone = Zone::new(1, "Test Zone", "A test zone.");
        zone.add_room(Room {
            id: 1,
            name: "Room One".to_string(),
            description: "First room.".to_string(),
            exits: HashMap::from([
                (Direction::North, RoomRef { zone_id: 1, room_id: 2 }),
            ]),
            fixtures: vec![],
            objects: vec![],
        });
        zone.add_room(Room {
            id: 2,
            name: "Room Two".to_string(),
            description: "Second room.".to_string(),
            exits: HashMap::from([
                (Direction::South, RoomRef { zone_id: 1, room_id: 1 }),
            ]),
            fixtures: vec![],
            objects: vec![],
        });
        world.add_zone(zone);
        world
    }

    #[test]
    fn get_existing_room() {
        assert!(make_world().get_room(1, 1).is_some());
    }

    #[test]
    fn get_missing_room_returns_none() {
        assert!(make_world().get_room(1, 99).is_none());
    }

    #[test]
    fn get_missing_zone_returns_none() {
        assert!(make_world().get_room(99, 1).is_none());
    }

    #[test]
    fn zone_ids_are_sorted() {
        let mut world = World::new();
        world.add_zone(Zone::new(3, "C", ""));
        world.add_zone(Zone::new(1, "A", ""));
        world.add_zone(Zone::new(2, "B", ""));
        assert_eq!(world.zone_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn validate_clean_world_returns_no_errors() {
        assert!(make_world().validate().is_empty());
    }

    #[test]
    fn validate_detects_dead_exit() {
        let mut world = World::new();
        let mut zone = Zone::new(1, "Zone", "");
        zone.add_room(Room {
            id: 1,
            name: "Room".to_string(),
            description: "".to_string(),
            exits: HashMap::from([
                (Direction::North, RoomRef { zone_id: 1, room_id: 999 }),
            ]),
            fixtures: vec![],
            objects: vec![],
        });
        world.add_zone(zone);
        assert!(!world.validate().is_empty());
    }
}
