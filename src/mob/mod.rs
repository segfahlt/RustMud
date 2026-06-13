pub mod core;
pub mod monster;
pub mod npc;
pub mod player;

pub use core::MobCore;
pub use monster::Monster;
pub use npc::Npc;
pub use player::Player;

use crate::world::RoomRef;

// The shared interface every mob type must implement.
// `core()` and `core_mut()` are required — everything else has a default
// implementation built on top of them, so concrete types get it for free.
pub trait Mobile {
    fn core(&self) -> &MobCore;
    fn core_mut(&mut self) -> &mut MobCore;

    fn name(&self) -> &str {
        &self.core().name
    }
    fn health(&self) -> u32 {
        self.core().health
    }
    fn location(&self) -> RoomRef {
        self.core().location
    }
    fn describe(&self);
}

// The enum that lets rooms and the world hold any mob type in one collection.
// Each variant wraps a concrete type — the type information is preserved.
pub enum Mob {
    Player(Player),
    Npc(Npc),
    Monster(Monster),
}

// Delegating Mobile impl: just forward each method to the inner type.
// The compiler ensures every Mob variant is handled — add a new variant
// and forget to update this, and it won't compile.
impl Mobile for Mob {
    fn core(&self) -> &MobCore {
        match self {
            Mob::Player(p) => p.core(),
            Mob::Npc(n) => n.core(),
            Mob::Monster(m) => m.core(),
        }
    }

    fn core_mut(&mut self) -> &mut MobCore {
        match self {
            Mob::Player(p) => p.core_mut(),
            Mob::Npc(n) => n.core_mut(),
            Mob::Monster(m) => m.core_mut(),
        }
    }

    fn describe(&self) {
        match self {
            Mob::Player(p) => p.describe(),
            Mob::Npc(n) => n.describe(),
            Mob::Monster(m) => m.describe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::RoomRef;

    fn loc() -> RoomRef { RoomRef { zone_id: 1, room_id: 1 } }
    fn player(name: &str) -> Player { Player::new(MobCore::new(1, name, 100, loc()), name) }
    fn monster(name: &str, hp: u32) -> Monster { Monster::new(MobCore::new(2, name, hp, loc()), false) }

    // --- Mobile trait via concrete types ---
    #[test] fn player_name()   { assert_eq!(player("Aldric").name(), "Aldric"); }
    #[test] fn player_health() { assert_eq!(player("Aldric").health(), 100); }

    // --- Mobile trait via Mob enum ---
    #[test]
    fn mob_enum_delegates_name() {
        let mob = Mob::Player(player("Aldric"));
        assert_eq!(mob.name(), "Aldric");
    }

    #[test]
    fn mob_enum_delegates_health() {
        let mob = Mob::Monster(monster("Troll", 80));
        assert_eq!(mob.health(), 80);
    }

    #[test]
    fn mob_enum_delegates_location() {
        let mob = Mob::Player(player("Aldric"));
        assert_eq!(mob.location().zone_id, 1);
        assert_eq!(mob.location().room_id, 1);
    }
}
