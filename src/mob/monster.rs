use crate::world::{MonsterTemplate, PlayerLocation};
use super::core::MobCore;
use super::Mobile;

pub struct MonsterInstance {
    pub id:            u32,
    pub template_id:   String,
    pub core:          MobCore,
    /// Client id or mob id of the current combat target.
    pub combat_target: Option<u32>,
    /// Location this instance returns to on respawn.
    pub spawn_point:   PlayerLocation,
    /// True while the mob is dead awaiting respawn.
    pub dead:          bool,
    /// Ticks remaining until respawn. 0 when alive or respawn disabled.
    pub respawn_ticks: u32,
}

impl MonsterInstance {
    pub fn spawn(id: u32, template: &MonsterTemplate, location: PlayerLocation) -> Self {
        let core = MobCore::new(id, &template.short, template.health_max, location);
        MonsterInstance {
            id,
            template_id:   template.id.clone(),
            core,
            combat_target: None,
            spawn_point:   location,
            dead:          false,
            respawn_ticks: 0,
        }
    }
}

impl Mobile for MonsterInstance {
    fn core(&self) -> &MobCore { &self.core }
    fn core_mut(&mut self) -> &mut MobCore { &mut self.core }

    fn describe(&self) -> String {
        format!("{} ({}/{} HP)", self.core.name, self.core.health, self.core.max_health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CombatStats, DamageType, HexCoord};

    fn loc() -> PlayerLocation {
        PlayerLocation::area(HexCoord::new(0, 0), 1)
    }

    fn make_template(id: &str, hp_min: u32, hp_max: u32) -> MonsterTemplate {
        MonsterTemplate {
            id:          id.to_string(),
            names:       vec![id.to_string()],
            short:       format!("a {id}"),
            description: format!("A {id}."),
            room_look:   format!("A {id} lurks here."),
            health_min:  hp_min,
            health_max:  hp_max,
            combat: CombatStats {
                attack: 5, defense: 2, damage_min: 1, damage_max: 4,
                attack_type: DamageType::Physical,
                xp_value: 10,
                resistances: vec![],
                immunities: vec![],
            },
            stationary: false, wanders: false, aggressive: false,
            follows_aggressive: false, calls_for_help: false,
            detection_range: 1, flee_threshold: 0,
            faction: None, respawn_secs: 60,
            chance_of_loot: 50, loot_table: vec![],
        }
    }

    #[test]
    fn spawn_sets_health_from_template_max() {
        let tmpl = make_template("creeper", 20, 40);
        let inst = MonsterInstance::spawn(1, &tmpl, loc());
        assert_eq!(inst.core.health, 40);
        assert_eq!(inst.core.max_health, 40);
    }

    #[test]
    fn spawn_copies_template_id() {
        let tmpl = make_template("creeper", 20, 40);
        let inst = MonsterInstance::spawn(1, &tmpl, loc());
        assert_eq!(inst.template_id, "creeper");
    }

    #[test]
    fn spawn_sets_name_from_short() {
        let tmpl = make_template("creeper", 20, 40);
        let inst = MonsterInstance::spawn(1, &tmpl, loc());
        assert_eq!(inst.core.name, "a creeper");
    }

    #[test]
    fn spawn_starts_alive_no_target() {
        let tmpl = make_template("creeper", 20, 40);
        let inst = MonsterInstance::spawn(1, &tmpl, loc());
        assert!(!inst.dead);
        assert!(inst.combat_target.is_none());
    }

    #[test]
    fn spawn_point_matches_location() {
        let tmpl = make_template("creeper", 20, 40);
        let inst = MonsterInstance::spawn(1, &tmpl, loc());
        assert_eq!(inst.spawn_point, loc());
    }

    #[test]
    fn mobile_trait_name() {
        let tmpl = make_template("thorn", 10, 10);
        let inst = MonsterInstance::spawn(2, &tmpl, loc());
        assert_eq!(inst.name(), "a thorn");
    }

    #[test]
    fn mobile_trait_health() {
        let tmpl = make_template("thorn", 10, 30);
        let inst = MonsterInstance::spawn(2, &tmpl, loc());
        assert_eq!(inst.health(), 30);
    }

    #[test]
    fn template_matches_name_prefix() {
        let tmpl = make_template("vine_creeper", 10, 10);
        assert!(tmpl.matches_name("vine"));
        assert!(!tmpl.matches_name("snake"));
    }
}
