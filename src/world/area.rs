use std::collections::HashMap;

use super::fixture::Fixture;
use super::hex::{AreaRef, EvolutionStage};
use super::object::{ObjectInstance, ObjectRegistry};
use super::room::Direction;

/// An outdoor location within a Zone, navigated via hex directions.
/// Areas are AI-generated and evolve over time via player traffic.
#[derive(Debug, Default)]
pub struct Area {
    pub id:          u32,
    pub name:        String,
    pub description: String,
    pub exits:       HashMap<Direction, AreaRef>,
    pub fixtures:    Vec<Fixture>,
    pub objects:     Vec<ObjectInstance>,

    // --- Evolution tracking ---
    pub evolution_stage: EvolutionStage,
    /// Hex grid stride used when this area was generated.
    pub stride:          u32,
    /// Hex grid offset used when this area was generated.
    pub offset:          u32,
    /// Cumulative player visits since area creation.
    pub visit_count:     u32,
    /// Visits within the current evolution window (resets on stage change).
    pub recent_visits:   u32,
    /// Set when visit thresholds are crossed; cleared after AI regeneration.
    pub refactor_pending: bool,
    /// False until the area has received its first AI-generated description.
    pub generated:        bool,
}

impl Area {
    pub fn render(&self, registry: &ObjectRegistry) -> String {
        let exits = if self.exits.is_empty() {
            "none".to_string()
        } else {
            let mut dirs: Vec<String> = self.exits.keys().map(|d| d.to_string()).collect();
            dirs.sort();
            dirs.join(", ")
        };

        let mut out = format!("[ {} ]\n{}", self.name, self.description);

        let mut extras = Vec::new();
        for fixture in &self.fixtures {
            let line = fixture.state_line();
            if !line.is_empty() {
                extras.push(line.to_string());
            }
        }
        for obj in &self.objects {
            extras.push(obj.room_look(registry).to_string());
        }
        if !extras.is_empty() {
            out.push('\n');
            for line in extras {
                out.push('\n');
                out.push_str(&line);
            }
        }

        out.push_str(&format!("\nExits: {}\n", exits));
        out
    }
}
