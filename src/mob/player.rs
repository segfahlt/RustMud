use super::core::MobCore;
use super::Mobile;

#[derive(Debug)]
pub struct Player {
    pub core:      MobCore,
    pub character_id: String,  // stable identity, survives game reboots
}

impl Player {
    pub fn new(core: MobCore, player_id: impl Into<String>) -> Self {
        Player { core, character_id: player_id.into() }
    }
}

impl Mobile for Player {
    fn core(&self) -> &MobCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut MobCore {
        &mut self.core
    }

    fn describe(&self) {
        println!(
            "[Player] {} ({}/{} HP)",
            self.core.name, self.core.health, self.core.max_health
        );
    }
}
