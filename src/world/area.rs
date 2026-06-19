use std::collections::HashMap;

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
    pub objects:     Vec<ObjectInstance>,
    /// Template IDs to spawn when the server starts. Used once during GameState init.
    pub mob_spawns:  Vec<String>,

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
    pub fn render(&self, registry: &ObjectRegistry, mob_lines: &[String]) -> String {
        let exits = if self.exits.is_empty() {
            "none".to_string()
        } else {
            let mut dirs: Vec<String> = self.exits.keys().map(|d| d.to_string()).collect();
            dirs.sort();
            dirs.join(", ")
        };

        let mut out = format!("[ {} ]\n{}", self.name, self.description);

        let mut extras: Vec<&str> = Vec::new();
        for obj in &self.objects {
            let line = obj.visible_line(registry);
            if !line.is_empty() {
                extras.push(line);
            }
        }
        let mob_refs: Vec<&str> = mob_lines.iter().map(|s| s.as_str()).collect();
        let all_extras: Vec<&str> = extras.into_iter().chain(mob_refs).collect();
        if !all_extras.is_empty() {
            out.push('\n');
            for line in all_extras {
                out.push('\n');
                out.push_str(line);
            }
        }

        out.push_str(&format!("\nExits: {}\n", exits));
        out
    }
}
