use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DamageType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    Physical,
    Toxic,
    Electric,
    Thermal,
    Coherence,
}

impl Default for DamageType {
    fn default() -> Self { DamageType::Physical }
}

// ---------------------------------------------------------------------------
// LootEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LootEntry {
    pub template_id: String,
    /// 0–100 probability this entry drops.
    pub chance:  u8,
    #[serde(default = "one")]
    pub qty_min: u32,
    #[serde(default = "one")]
    pub qty_max: u32,
}

fn one() -> u32 { 1 }

// ---------------------------------------------------------------------------
// CombatStats
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CombatStats {
    pub attack:     u32,
    pub defense:    u32,
    pub damage_min: u32,
    pub damage_max: u32,
    #[serde(default)]
    pub attack_type: DamageType,
    pub xp_value:   u32,
    #[serde(default)]
    pub resistances: Vec<DamageType>,
    #[serde(default)]
    pub immunities:  Vec<DamageType>,
}

// ---------------------------------------------------------------------------
// MonsterTemplate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonsterTemplate {
    pub id:          String,
    /// Match names used for targeting — first is canonical.
    pub names:       Vec<String>,
    /// Short form shown in inventory / combat messages.
    pub short:       String,
    /// Full text shown on `examine <mob>`.
    pub description: String,
    /// One-liner shown when the mob is present in a room or area.
    pub room_look:   String,
    pub health_min:  u32,
    pub health_max:  u32,
    pub combat:      CombatStats,

    // --- Behaviour flags ---
    /// Cannot move — plants, sessile alien organisms.
    #[serde(default)]
    pub stationary:         bool,
    /// Moves between adjacent areas/rooms on tick.
    #[serde(default)]
    pub wanders:            bool,
    /// Attacks players on sight.
    #[serde(default)]
    pub aggressive:         bool,
    /// Pursues fleeing players across locations.
    #[serde(default)]
    pub follows_aggressive: bool,
    /// Alerts same-template mobs in the same location when attacked.
    #[serde(default)]
    pub calls_for_help:     bool,
    /// How many locations away the mob detects players (1 = current only).
    #[serde(default = "default_detection_range")]
    pub detection_range:    u8,
    /// HP% at which this mob tries to flee. 0 = never flees.
    #[serde(default)]
    pub flee_threshold:     u8,

    // --- World state ---
    #[serde(default)]
    pub faction:        Option<String>,
    /// Seconds before respawning at spawn point. 0 = no respawn.
    #[serde(default)]
    pub respawn_secs:   u32,

    // --- Loot ---
    /// 0–100 chance any loot drops on death.
    #[serde(default)]
    pub chance_of_loot: u8,
    #[serde(default)]
    pub loot_table:     Vec<LootEntry>,
}

fn default_detection_range() -> u8 { 1 }

impl MonsterTemplate {
    pub fn matches_name(&self, input: &str) -> bool {
        self.names.iter().any(|n| n.starts_with(input))
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

pub type MobRegistry = HashMap<String, MonsterTemplate>;
