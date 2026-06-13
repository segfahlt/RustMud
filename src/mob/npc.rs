use super::core::MobCore;
use super::Mobile;

#[derive(Debug)]
pub struct Npc {
    pub core: MobCore,
    // Future: dialogue trees, faction, shop inventory, etc.
}

impl Npc {
    pub fn new(core: MobCore) -> Self {
        Npc { core }
    }
}

impl Mobile for Npc {
    fn core(&self) -> &MobCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut MobCore {
        &mut self.core
    }

    fn describe(&self) {
        println!(
            "[NPC] {} ({}/{} HP)",
            self.core.name, self.core.health, self.core.max_health
        );
    }
}
