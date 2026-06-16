use crate::world::{ObjectInstance, PlayerLocation};
use super::core::MobCore;
use super::Mobile;

#[derive(Debug)]
pub struct Player {
    pub core:         MobCore,
    pub character_id: String,
    pub inventory:    Vec<ObjectInstance>,
    /// The outdoor Area the player last entered a building from.
    /// Used to return them to the right spot when they exit.
    pub last_area:    Option<PlayerLocation>,
    /// Cached from session permissions — controls admin-only display (e.g. room IDs).
    pub is_admin:     bool,
}

impl Player {
    pub fn new(core: MobCore, player_id: impl Into<String>) -> Self {
        Player {
            core,
            character_id: player_id.into(),
            inventory:    vec![],
            last_area:    None,
            is_admin:     false,
        }
    }
}

impl Mobile for Player {
    fn core(&self) -> &MobCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut MobCore {
        &mut self.core
    }

    fn describe(&self) -> String {
        format!(
            "[Player] {} ({}/{} HP)",
            self.core.name, self.core.health, self.core.max_health
        )
    }
}
