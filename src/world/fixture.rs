use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FixtureCategory {
    Structural,
    Container,
    CraftingStation,
    Environmental,
    Toggle,
    Commerce,
    Coherence,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FixtureState {
    pub current: String,
}

impl Default for FixtureState {
    fn default() -> Self {
        FixtureState { current: "default".to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Fixture {
    pub id:       String,
    /// Names used for player targeting: `look at forge`, `examine notice board`.
    pub names:    Vec<String>,
    pub category: FixtureCategory,
    /// State key → one-liner shown at room entry. Key "default" is the fallback.
    pub state_lines: HashMap<String, String>,
    /// Text shown on `look at <fixture>`.
    pub look:     String,
    /// Text shown on `examine <fixture>` — more detail than look.
    pub examine:  String,
    #[serde(default)]
    pub read:     Option<String>,
    #[serde(default)]
    pub state:    FixtureState,
    /// Persist state across reboots (crafting stations, toggles that matter).
    #[serde(default)]
    pub persist_state: bool,
    /// If true, state key is the global Coherence threat level instead of fixture.state.current.
    #[serde(default)]
    pub coherence_driven: bool,
}

impl Fixture {
    /// One-liner to display at room entry/look. Falls back from current state → "default" → "".
    pub fn state_line(&self) -> &str {
        self.state_lines.get(&self.state.current)
            .or_else(|| self.state_lines.get("default"))
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Returns true if the input matches one of this fixture's names (prefix match).
    pub fn matches_name(&self, input: &str) -> bool {
        self.names.iter().any(|n| n.starts_with(input))
    }
}
