use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObjectCategory {
    Weapon,
    Armor,
    Tool,
    Consumable,
    Component,
    Container,
    Data,
    Currency,
    TradeGood,
    Quest,
    Bonded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObjectFlag {
    NoDrop,
    NoSell,
    NoGive,
    NoTrade,
    EarthOrigin,
    Quest,
    Stackable,
    Consumable,
    Container,
    TwoHanded,
    LightSource,
    Hidden,
    Bonded,
    CorporateIssue,
    SettlerMade,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Weight {
    Tiny,
    Light,
    Medium,
    Heavy,
    Bulky,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Pristine,
    Good,
    Worn,
    Damaged,
    Broken,
}

impl Condition {
    pub fn label(&self) -> &'static str {
        match self {
            Condition::Pristine => "pristine",
            Condition::Good     => "good",
            Condition::Worn     => "worn",
            Condition::Damaged  => "damaged",
            Condition::Broken   => "broken",
        }
    }
}

impl Default for Condition {
    fn default() -> Self { Condition::Good }
}

/// Defines a category of object. All instances of this template share these base properties.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObjectTemplate {
    pub id:          String,
    /// Match names — first is canonical. Used for `get`, `examine`, etc.
    pub names:       Vec<String>,
    /// Short form shown in inventory: "a hunting knife".
    pub short:       String,
    /// One-liner shown when object is on the floor of a room.
    pub room_look:   String,
    /// Shown on `look at <obj>` or `examine <obj>`.
    pub description: String,
    pub category:    ObjectCategory,
    pub weight:      Weight,
    #[serde(default)]
    pub flags:       Vec<ObjectFlag>,
    #[serde(default)]
    pub value:       u32,
}

impl ObjectTemplate {
    pub fn matches_name(&self, input: &str) -> bool {
        self.names.iter().any(|n| n.starts_with(input))
    }
}

/// A specific copy of an ObjectTemplate that exists in the world at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInstance {
    pub id:          String,
    pub template_id: String,
    #[serde(default)]
    pub condition:   Condition,
    #[serde(default)]
    pub custom_name: Option<String>,
    #[serde(default)]
    pub custom_desc: Option<String>,
}

impl ObjectInstance {
    pub fn new(template_id: impl Into<String>) -> Self {
        ObjectInstance {
            id:          Uuid::new_v4().to_string(),
            template_id: template_id.into(),
            condition:   Condition::Good,
            custom_name: None,
            custom_desc: None,
        }
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn short<'a>(&'a self, registry: &'a ObjectRegistry) -> &'a str {
        self.custom_name.as_deref()
            .or_else(|| registry.get(&self.template_id).map(|t| t.short.as_str()))
            .unwrap_or("an unknown object")
    }

    pub fn room_look<'a>(&'a self, registry: &'a ObjectRegistry) -> &'a str {
        registry.get(&self.template_id)
            .map(|t| t.room_look.as_str())
            .unwrap_or("An object lies here.")
    }

    pub fn description<'a>(&'a self, registry: &'a ObjectRegistry) -> &'a str {
        if let Some(desc) = &self.custom_desc {
            return desc.as_str();
        }
        registry.get(&self.template_id)
            .map(|t| t.description.as_str())
            .unwrap_or("You see nothing special.")
    }
}

/// Global lookup: template_id → ObjectTemplate. Built at load time, read-only during play.
pub type ObjectRegistry = HashMap<String, ObjectTemplate>;
