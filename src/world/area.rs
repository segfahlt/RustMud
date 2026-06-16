use std::collections::HashMap;

use super::fixture::Fixture;
use super::hex::AreaRef;
use super::object::{ObjectInstance, ObjectRegistry};
use super::room::Direction;

/// An outdoor location within a Zone, navigated via hex directions.
/// Areas are AI-generated and evolve over time via player traffic.
#[derive(Debug)]
pub struct Area {
    pub id:          u32,
    pub name:        String,
    pub description: String,
    pub exits:       HashMap<Direction, AreaRef>,
    pub fixtures:    Vec<Fixture>,
    pub objects:     Vec<ObjectInstance>,
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
