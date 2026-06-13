use std::collections::HashMap;
use super::room::Room;

#[derive(Debug)]
pub struct Zone {
    pub id: u32,
    pub name: String,
    pub description: String,
    rooms: HashMap<u32, Room>,
}

impl Zone {
    pub fn new(id: u32, name: impl Into<String>, description: impl Into<String>) -> Self {
        Zone {
            id,
            name: name.into(),
            description: description.into(),
            rooms: HashMap::new(),
        }
    }

    pub fn add_room(&mut self, room: Room) {
        self.rooms.insert(room.id, room);
    }

    pub fn get_room(&self, room_id: u32) -> Option<&Room> {
        self.rooms.get(&room_id)
    }

    // Returns room IDs in sorted order — callers don't need to touch the HashMap.
    pub fn room_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.rooms.keys().copied().collect();
        ids.sort();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Room;
    use std::collections::HashMap;

    fn make_room(id: u32) -> Room {
        Room { id, name: format!("Room {id}"), description: String::new(), exits: HashMap::new() }
    }

    #[test]
    fn add_and_get_room() {
        let mut zone = Zone::new(1, "Test", "");
        zone.add_room(make_room(1));
        assert!(zone.get_room(1).is_some());
    }

    #[test]
    fn get_missing_room_returns_none() {
        assert!(Zone::new(1, "Test", "").get_room(99).is_none());
    }

    #[test]
    fn room_ids_are_sorted() {
        let mut zone = Zone::new(1, "Test", "");
        zone.add_room(make_room(3));
        zone.add_room(make_room(1));
        zone.add_room(make_room(2));
        assert_eq!(zone.room_ids(), vec![1, 2, 3]);
    }
}
