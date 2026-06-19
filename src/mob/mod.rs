pub mod core;
pub mod monster;
pub mod npc;
pub mod player;

pub use core::MobCore;
pub use monster::MonsterInstance;
pub use npc::Npc;
pub use player::{Equipment, Player};

use crate::world::PlayerLocation;

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
    fn location(&self) -> PlayerLocation {
        self.core().location
    }
    fn describe(&self) -> String;
}

pub enum Mob {
    Player(Player),
    Npc(Npc),
    Monster(MonsterInstance),
}

impl Mobile for Mob {
    fn core(&self) -> &MobCore {
        match self {
            Mob::Player(p)  => p.core(),
            Mob::Npc(n)     => n.core(),
            Mob::Monster(m) => m.core(),
        }
    }

    fn core_mut(&mut self) -> &mut MobCore {
        match self {
            Mob::Player(p)  => p.core_mut(),
            Mob::Npc(n)     => n.core_mut(),
            Mob::Monster(m) => m.core_mut(),
        }
    }

    fn describe(&self) -> String {
        match self {
            Mob::Player(p)  => p.describe(),
            Mob::Npc(n)     => n.describe(),
            Mob::Monster(m) => m.describe(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::world::{AreaRef, HexCoord};

    fn loc() -> PlayerLocation {
        PlayerLocation::area(HexCoord::new(0, 1), 1)
    }

    fn player(name: &str) -> Player { Player::new(MobCore::new(1, name, 100, loc()), name) }
    fn monster(name: &str, hp: u32) -> MonsterInstance {
        use crate::world::{CombatStats, DamageType, MonsterTemplate};
        let tmpl = MonsterTemplate {
            id: name.to_lowercase(), names: vec![name.to_lowercase()],
            short: name.to_string(), description: String::new(), room_look: String::new(),
            health_min: hp, health_max: hp,
            combat: CombatStats {
                attack: 5, defense: 0, damage_min: 1, damage_max: 3,
                attack_type: DamageType::Physical, xp_value: 10,
                resistances: vec![], immunities: vec![],
            },
            stationary: false, wanders: false, aggressive: false,
            follows_aggressive: false, calls_for_help: false,
            detection_range: 1, flee_threshold: 0,
            faction: None, respawn_secs: 60, chance_of_loot: 0, loot_table: vec![],
            food_chain_tier: crate::world::FoodChainTier::Grazer,
            generated: false,
        };
        MonsterInstance::spawn(2, &tmpl, loc())
    }

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
        let area_ref = AreaRef { zone: HexCoord::new(0, 1), area_id: 1 };
        assert_eq!(mob.location().as_area_ref(), Some(area_ref));
    }
}
