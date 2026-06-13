use crate::world::RoomRef;

// Shared data carried by every mob, regardless of type.
// Each concrete mob type (Player, Npc, Monster) embeds one of these.
#[derive(Debug)]
pub struct MobCore {
    pub id: u32,
    pub name: String,
    pub health: u32,
    pub max_health: u32,
    pub location: RoomRef,
}

impl MobCore {
    pub fn new(id: u32, name: impl Into<String>, health: u32, location: RoomRef) -> Self {
        MobCore {
            id,
            name: name.into(),
            health,
            max_health: health,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::RoomRef;

    fn loc() -> RoomRef { RoomRef { zone_id: 1, room_id: 1 } }

    #[test]
    fn new_sets_max_health_equal_to_health() {
        let core = MobCore::new(1, "Test", 50, loc());
        assert_eq!(core.health, 50);
        assert_eq!(core.max_health, 50);
    }

    #[test]
    fn new_stores_name() {
        let core = MobCore::new(1, "Gandalf", 100, loc());
        assert_eq!(core.name, "Gandalf");
    }

    #[test]
    fn new_accepts_string_slice() {
        let name = String::from("Owned");
        let core = MobCore::new(1, name, 10, loc());
        assert_eq!(core.name, "Owned");
    }
}
