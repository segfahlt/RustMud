use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use super::hex::ExitDestination;
use super::object::{ObjectInstance, ObjectRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
    Up,
    Down,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::North     => "north",
            Direction::South     => "south",
            Direction::East      => "east",
            Direction::West      => "west",
            Direction::Up        => "up",
            Direction::Down      => "down",
            Direction::NorthEast => "northeast",
            Direction::NorthWest => "northwest",
            Direction::SouthEast => "southeast",
            Direction::SouthWest => "southwest",
        })
    }
}

// FromStr lets us write `"north".parse::<Direction>()`.
// The loader uses this to convert JSON string keys into Direction values.
// Err type is () because callers always substitute their own error context.
impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "north"     => Ok(Direction::North),
            "south"     => Ok(Direction::South),
            "east"      => Ok(Direction::East),
            "west"      => Ok(Direction::West),
            "up"        => Ok(Direction::Up),
            "down"      => Ok(Direction::Down),
            "northeast" => Ok(Direction::NorthEast),
            "northwest" => Ok(Direction::NorthWest),
            "southeast" => Ok(Direction::SouthEast),
            "southwest" => Ok(Direction::SouthWest),
            _           => Err(()),
        }
    }
}

/// A globally registered indoor location: settlement rooms, buildings, dungeons.
/// Rooms are human-authored, identified by a server-assigned sequential ID.
/// Builder sets breadcrumb_zone and breadcrumb_building for location display.
#[derive(Debug)]
pub struct Room {
    pub id:                   u32,
    pub name:                 String,
    pub description:          String,
    pub breadcrumb_zone:      String,
    pub breadcrumb_building:  String,
    pub exits:                HashMap<Direction, ExitDestination>,
    pub objects:              Vec<ObjectInstance>,
}

impl Room {
    pub fn render(&self, registry: &ObjectRegistry, is_admin: bool) -> String {
        let exits = if self.exits.is_empty() {
            "none".to_string()
        } else {
            let mut dirs: Vec<String> = self.exits.keys().map(|d| d.to_string()).collect();
            dirs.sort();
            dirs.join(", ")
        };

        let crumb = if self.breadcrumb_building.is_empty() {
            format!("{{Y}}[ {} > {} ]{{/}}", self.breadcrumb_zone, self.name)
        } else {
            format!("{{Y}}[ {} > {} > {} ]{{/}}", self.breadcrumb_zone, self.breadcrumb_building, self.name)
        };
        let header = if is_admin {
            format!("{} {{K}}#{}{{/}}", crumb, self.id)
        } else {
            crumb
        };

        let mut out = format!("{}\n{}", header, self.description);

        let mut extras: Vec<&str> = Vec::new();
        for obj in &self.objects {
            let line = obj.visible_line(registry);
            if !line.is_empty() {
                extras.push(line);
            }
        }
        if !extras.is_empty() {
            out.push('\n');
            for line in extras {
                out.push('\n');
                out.push_str(line);
            }
        }

        out.push_str(&format!("\n{{c}}Exits:{{/}} {}\n", exits));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_from_str_valid() {
        assert_eq!("north".parse::<Direction>().unwrap(),     Direction::North);
        assert_eq!("south".parse::<Direction>().unwrap(),     Direction::South);
        assert_eq!("east".parse::<Direction>().unwrap(),      Direction::East);
        assert_eq!("west".parse::<Direction>().unwrap(),      Direction::West);
        assert_eq!("up".parse::<Direction>().unwrap(),        Direction::Up);
        assert_eq!("down".parse::<Direction>().unwrap(),      Direction::Down);
        assert_eq!("northeast".parse::<Direction>().unwrap(), Direction::NorthEast);
        assert_eq!("northwest".parse::<Direction>().unwrap(), Direction::NorthWest);
        assert_eq!("southeast".parse::<Direction>().unwrap(), Direction::SouthEast);
        assert_eq!("southwest".parse::<Direction>().unwrap(), Direction::SouthWest);
    }

    #[test]
    fn direction_from_str_invalid() {
        assert!("sideways".parse::<Direction>().is_err());
        assert!("North".parse::<Direction>().is_err()); // case-sensitive at this level
        assert!("".parse::<Direction>().is_err());
    }

    fn make_room(id: u32) -> Room {
        Room {
            id,
            name:                "Test Room".to_string(),
            description:         "A plain room.".to_string(),
            breadcrumb_zone:     "Zone".to_string(),
            breadcrumb_building: "Building".to_string(),
            exits:               HashMap::new(),
            objects:             vec![],
        }
    }

    #[test]
    fn render_non_admin_no_id() {
        let room = make_room(7);
        let out = room.render(&HashMap::new(), false);
        assert!(out.contains("Test Room"));
        assert!(!out.contains("#7"));
    }

    #[test]
    fn render_admin_shows_id() {
        let room = make_room(7);
        let out = room.render(&HashMap::new(), true);
        assert!(out.contains("#7"));
    }

    #[test]
    fn render_includes_exits_label() {
        let room = make_room(1);
        let out = room.render(&HashMap::new(), false);
        assert!(out.contains("Exits:"));
    }

    #[test]
    fn direction_display_diagonals() {
        assert_eq!(Direction::NorthEast.to_string(), "northeast");
        assert_eq!(Direction::NorthWest.to_string(), "northwest");
        assert_eq!(Direction::SouthEast.to_string(), "southeast");
        assert_eq!(Direction::SouthWest.to_string(), "southwest");
    }
}
