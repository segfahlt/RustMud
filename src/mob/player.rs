use serde::{Deserialize, Serialize};

use crate::world::{ObjectInstance, ObjectRegistry, PlayerLocation};
use crate::world::object::EquipSlot;
use super::core::MobCore;
use super::Mobile;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Equipment {
    #[serde(default)]
    pub main_hand: Option<ObjectInstance>,
    #[serde(default)]
    pub off_hand:  Option<ObjectInstance>,
    #[serde(default)]
    pub body:      Option<ObjectInstance>,
    #[serde(default)]
    pub head:      Option<ObjectInstance>,
    #[serde(default)]
    pub hands:     Option<ObjectInstance>,
    #[serde(default)]
    pub feet:      Option<ObjectInstance>,
}

impl Equipment {
    pub fn slot(&self, slot: EquipSlot) -> &Option<ObjectInstance> {
        match slot {
            EquipSlot::MainHand => &self.main_hand,
            EquipSlot::OffHand  => &self.off_hand,
            EquipSlot::Body     => &self.body,
            EquipSlot::Head     => &self.head,
            EquipSlot::Hands    => &self.hands,
            EquipSlot::Feet     => &self.feet,
        }
    }

    pub fn slot_mut(&mut self, slot: EquipSlot) -> &mut Option<ObjectInstance> {
        match slot {
            EquipSlot::MainHand => &mut self.main_hand,
            EquipSlot::OffHand  => &mut self.off_hand,
            EquipSlot::Body     => &mut self.body,
            EquipSlot::Head     => &mut self.head,
            EquipSlot::Hands    => &mut self.hands,
            EquipSlot::Feet     => &mut self.feet,
        }
    }

    /// Returns the slot containing the first item whose name matches `input`.
    pub fn find_equipped(&self, input: &str, registry: &ObjectRegistry) -> Option<EquipSlot> {
        for slot in EquipSlot::all() {
            if let Some(obj) = self.slot(slot) {
                if registry.get(&obj.template_id).map(|t| t.matches_name(input)).unwrap_or(false) {
                    return Some(slot);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Player {
    pub core:         MobCore,
    pub character_id: String,
    pub inventory:    Vec<ObjectInstance>,
    pub equipment:    Equipment,
    /// The outdoor Area the player last entered a building from.
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
            equipment:    Equipment::default(),
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
