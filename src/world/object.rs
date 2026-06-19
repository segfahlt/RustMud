use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::hex::EvolutionStage;

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObjectCategory {
    // Carriable items
    Weapon,
    Armor,
    Tool,
    Consumable,
    Component,   // crafting material / stackable resource
    Container,   // carried container (backpack, bag)
    Data,        // notes, books, data chips — primarily read
    Currency,
    TradeGood,
    Quest,
    Bonded,      // soul-bound / unique personal item
    // Fixed-in-place (formerly Fixture)
    Structural,       // walls, gates, doors, fences
    CraftingStation,  // forge, workbench, terminal
    Environmental,    // plants, terrain features, ambient scenery
    Toggle,           // switches, levers, control panels
    Commerce,         // vendor terminals, trading posts
    Coherence,        // alien growths, coherence emitters
}

impl ObjectCategory {
    pub fn is_fixture(&self) -> bool {
        matches!(self,
            ObjectCategory::Structural
            | ObjectCategory::CraftingStation
            | ObjectCategory::Environmental
            | ObjectCategory::Toggle
            | ObjectCategory::Commerce
            | ObjectCategory::Coherence
        )
    }
}

// ---------------------------------------------------------------------------
// EquipSlot  (which slot on the body an item occupies when equipped)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    MainHand,
    OffHand,
    Body,
    Head,
    Hands,
    Feet,
}

impl EquipSlot {
    pub fn all() -> [EquipSlot; 6] {
        [EquipSlot::MainHand, EquipSlot::OffHand, EquipSlot::Body,
         EquipSlot::Head, EquipSlot::Hands, EquipSlot::Feet]
    }

    pub fn label(self) -> &'static str {
        match self {
            EquipSlot::MainHand => "main hand",
            EquipSlot::OffHand  => "off hand",
            EquipSlot::Body     => "body",
            EquipSlot::Head     => "head",
            EquipSlot::Hands    => "hands",
            EquipSlot::Feet     => "feet",
        }
    }
}

// ---------------------------------------------------------------------------
// FixturePermanence  (moved here from fixture.rs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FixturePermanence {
    /// Blocks Area devolution below minimum_stage.
    Permanent,
    /// Can be removed by devolution without dissonance.
    Degradable,
}

impl Default for FixturePermanence {
    fn default() -> Self { FixturePermanence::Degradable }
}

// ---------------------------------------------------------------------------
// Flags  (composable boolean traits)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObjectFlag {
    // Behaviour restrictions
    NoDrop,
    NoSell,
    NoGive,
    NoTrade,
    Bonded,       // bound to the character who picks it up

    // Provenance / origin
    EarthOrigin,      // manufactured on Earth before the mission
    CorporateIssue,   // standard Corporate kit, may be Earth or colony-made
    SettlerMade,      // crafted by colonists
    AlienMade,        // alien artefact or alien-manufactured
    Salvaged,         // recovered from wreckage or debris

    // Properties
    Stackable,        // can stack in inventory (components, currency)
    TwoHanded,        // requires both hands (weapons, large tools)
    LightSource,      // emits light when active
    Perishable,       // degrades over time (food, medicine, organic)
    Restricted,       // requires Corporate authorisation to possess/use
    Hidden,           // not visible in room_look without searching
    Quest,            // marks a key story item
}

// ---------------------------------------------------------------------------
// Weight  (encumbrance — how much it burdens the carrier)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Weight {
    Tiny,    // coin, small tool, folded note
    Light,   // knife, comm unit, paperback
    Medium,  // medkit, short sword, full canteen
    Heavy,   // rifle, armour plate, power cell
}

impl Default for Weight {
    fn default() -> Self { Weight::Light }
}

// ---------------------------------------------------------------------------
// Bulk  (physical size — how much space it takes in a container or hand)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Bulk {
    Tiny,    // fits in a pocket: coins, pills, data chips
    Small,   // fits in a bag: knife, book, comm unit
    Medium,  // takes a slot: medkit, short sword, lantern
    Large,   // needs a backpack or both arms: rifle, sleeping bag, crate
    Huge,    // barely portable: generator, crate of supplies
}

impl Default for Bulk {
    fn default() -> Self { Bulk::Small }
}

// ---------------------------------------------------------------------------
// Material  (what it is made of — drives salvage, repair, coherence effects)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    Metal,         // ferrous or alloy — salvageable at the forge
    Composite,     // human-engineered mixed materials
    Fabric,        // cloth, synthetic fibre, woven materials
    Organic,       // biological, Earth-origin (leather, wood, paper, food)
    AlienOrganic,  // biological, alien-origin — coherence-sensitive
    Electronic,    // circuit boards, sensors, comm hardware
    Paper,         // documents, books, printed media
    Ceramic,       // high-heat, brittle, often structural
    Crystal,       // alien mineral formations — coherence-sensitive
    Unknown,       // unidentified material
}

impl Default for Material {
    fn default() -> Self { Material::Unknown }
}

// ---------------------------------------------------------------------------
// Condition  (current state of the instance)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// ObjectTemplate  (the blueprint — shared by all instances of this object)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObjectTemplate {
    pub id:          String,
    /// Match names — first is canonical. Used for `get`, `examine`, etc.
    pub names:       Vec<String>,
    /// Short form shown in inventory listings: "a hunting knife".
    #[serde(default)]
    pub short:       String,
    /// One-liner shown when a carriable object is on the floor. Fixtures use state_lines instead.
    #[serde(default)]
    pub room_look:   String,
    /// Full text shown on `examine <obj>`.
    pub description: String,
    /// Text shown on `read <obj>` (Data category). Falls back to description if absent.
    #[serde(default)]
    pub read:        Option<String>,
    pub category:    ObjectCategory,
    #[serde(default)]
    pub weight:      Weight,
    /// Physical volume — how much space it occupies in a container or hand.
    #[serde(default)]
    pub bulk:        Bulk,
    /// What the object is made of — drives salvage, repair, and coherence interactions.
    #[serde(default)]
    pub material:    Material,
    #[serde(default)]
    pub flags:       Vec<ObjectFlag>,
    #[serde(default)]
    pub value:       u32,
    /// Which equipment slot this item occupies. Required for Armor; Weapon → main_hand via `wield`.
    #[serde(default)]
    pub equip_slot:  Option<EquipSlot>,

    // --- Fixture-specific fields (None / false for regular carriable objects) ---

    /// State key → one-liner shown at room entry. Key "default" is the fallback.
    /// If set, this object is treated as a fixed fixture (not pickable).
    #[serde(default)]
    pub state_lines:      Option<HashMap<String, String>>,
    /// Whether this fixture blocks Area devolution.
    #[serde(default)]
    pub permanence:       Option<FixturePermanence>,
    /// Permanent fixtures set this to the lowest EvolutionStage the Area may devolve to.
    #[serde(default)]
    pub minimum_stage:    Option<EvolutionStage>,
    /// If set, entering this fixture moves the player into the given Room.
    #[serde(default)]
    pub connects_to_room: Option<u32>,
    /// If set, auto-entry only triggers when the player moves in this direction.
    #[serde(default)]
    pub direction:        Option<String>,
    /// If true, the state key is driven by global Coherence level instead of instance state.
    #[serde(default)]
    pub coherence_driven: bool,
    /// Persist state across reboots (crafting stations, toggles that matter).
    #[serde(default)]
    pub persist_state:    bool,

    // --- Consumable fields ---

    /// HP restored when consumed. 0 = no healing (purely flavour or side-effect).
    #[serde(default)]
    pub health_restore:  u32,
    /// Message shown to the player on consume. Defaults to a generic "You consume <short>."
    #[serde(default)]
    pub consume_message: Option<String>,
}

impl ObjectTemplate {
    pub fn matches_name(&self, input: &str) -> bool {
        self.names.iter().any(|n| n.starts_with(input))
    }

    pub fn is_stackable(&self) -> bool {
        self.flags.contains(&ObjectFlag::Stackable)
    }
}

// ---------------------------------------------------------------------------
// ObjectInstance  (a single copy of a template that exists in the world)
// ---------------------------------------------------------------------------

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
    /// Current state key for stateful fixtures (e.g. "open", "closed"). None = "default".
    #[serde(default)]
    pub state:       Option<String>,
    /// Stack size. Always 1 for non-stackable items. Stackable items merge on pickup.
    #[serde(default = "default_quantity")]
    pub quantity:    u32,
}

fn default_quantity() -> u32 { 1 }

impl ObjectInstance {
    pub fn new(template_id: impl Into<String>) -> Self {
        ObjectInstance {
            id:          Uuid::new_v4().to_string(),
            template_id: template_id.into(),
            condition:   Condition::Good,
            custom_name: None,
            custom_desc: None,
            state:       None,
            quantity:    1,
        }
    }

    pub fn new_stack(template_id: impl Into<String>, quantity: u32) -> Self {
        ObjectInstance { quantity, ..Self::new(template_id) }
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

    /// The line shown when this object is present in a room.
    /// Fixtures use state_lines (keyed by instance state); carriable objects use room_look.
    pub fn visible_line<'a>(&'a self, registry: &'a ObjectRegistry) -> &'a str {
        let tmpl = match registry.get(&self.template_id) {
            Some(t) => t,
            None    => return "",
        };
        if let Some(state_lines) = &tmpl.state_lines {
            let key = self.state.as_deref().unwrap_or("default");
            return state_lines.get(key)
                .or_else(|| state_lines.get("default"))
                .map(|s| s.as_str())
                .unwrap_or("");
        }
        tmpl.room_look.as_str()
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

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Global lookup: template_id → ObjectTemplate. Built at load time, read-only during play.
pub type ObjectRegistry = HashMap<String, ObjectTemplate>;
