use std::collections::HashMap;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::fixture::Fixture;
use super::object::{ObjectInstance, ObjectRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

// FromStr lets us write `"north".parse::<Direction>()`.
// The loader uses this to convert JSON string keys into Direction values.
// Err type is () because callers always substitute their own error context.
impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "north" => Ok(Direction::North),
            "south" => Ok(Direction::South),
            "east"  => Ok(Direction::East),
            "west"  => Ok(Direction::West),
            "up"    => Ok(Direction::Up),
            "down"  => Ok(Direction::Down),
            _       => Err(()),
        }
    }
}

// Deserialize added so serde can read RoomRef values from JSON exit objects.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct RoomRef {
    pub zone_id: u32,
    pub room_id: u32,
}

#[derive(Debug)]
pub struct Room {
    pub id:          u32,
    pub name:        String,
    pub description: String,
    pub exits:       HashMap<Direction, RoomRef>,
    pub fixtures:    Vec<Fixture>,
    pub objects:     Vec<ObjectInstance>,
}

impl Room {
    pub fn render(&self, registry: &ObjectRegistry) -> String {
        let exits = if self.exits.is_empty() {
            "none".to_string()
        } else {
            let mut dirs: Vec<String> = self.exits
                .keys()
                .map(|dir| format!("{:?}", dir).to_lowercase())
                .collect();
            dirs.sort();
            dirs.join(", ")
        };

        let mut out = format!("[ {} ]\n{}", self.name, self.description);

        // Fixture state lines and room object lines go between description and exits.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_from_str_valid() {
        assert_eq!("north".parse::<Direction>().unwrap(), Direction::North);
        assert_eq!("south".parse::<Direction>().unwrap(), Direction::South);
        assert_eq!("east".parse::<Direction>().unwrap(),  Direction::East);
        assert_eq!("west".parse::<Direction>().unwrap(),  Direction::West);
        assert_eq!("up".parse::<Direction>().unwrap(),    Direction::Up);
        assert_eq!("down".parse::<Direction>().unwrap(),  Direction::Down);
    }

    #[test]
    fn direction_from_str_invalid() {
        assert!("sideways".parse::<Direction>().is_err());
        assert!("North".parse::<Direction>().is_err()); // case-sensitive at this level
        assert!("".parse::<Direction>().is_err());
    }
}
