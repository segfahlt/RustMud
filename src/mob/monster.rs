use super::core::MobCore;
use super::Mobile;

#[derive(Debug)]
pub struct Monster {
    pub core: MobCore,
    pub aggressive: bool, // attacks players on sight
    // Future: loot table, spawn point, behavior scripts, etc.
}

impl Monster {
    pub fn new(core: MobCore, aggressive: bool) -> Self {
        Monster { core, aggressive }
    }
}

impl Mobile for Monster {
    fn core(&self) -> &MobCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut MobCore {
        &mut self.core
    }

    fn describe(&self) -> String {
        let aggro = if self.aggressive { " (aggressive)" } else { "" };
        format!(
            "[Monster] {} ({}/{} HP){}",
            self.core.name, self.core.health, self.core.max_health, aggro
        )
    }
}
