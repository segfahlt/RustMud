use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::room::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

/// Reference to an outdoor Area within a Zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct AreaRef {
    pub zone:    HexCoord,
    pub area_id: u32,
}

/// Points back to the Permanent fixture that owns an Area→Room gateway.
/// Stored in Room exits so the engine resolves which Area to return the player to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FixtureRef {
    pub zone:       HexCoord,
    pub area_id:    u32,
    pub fixture_id: String,
}

/// Destination of a Room exit: another Room or back to an Area via its gateway fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExitDestination {
    Room { room_id: u32 },
    Fixture(FixtureRef),
}

/// Area evolution stages, driven by visitor traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvolutionStage {
    #[default]
    Pristine,
    Marked,
    Path,
    Footpath,
    Trail,
    Road,
}

/// Runtime player position — either in an outdoor Area or inside a Room cluster.
/// Copy because all inner types are Copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PlayerLocation {
    Area { zone_q: i32, zone_r: i32, area_id: u32 },
    Room { room_id: u32 },
}

impl PlayerLocation {
    pub fn area(zone: HexCoord, area_id: u32) -> Self {
        PlayerLocation::Area { zone_q: zone.q, zone_r: zone.r, area_id }
    }

    pub fn room(room_id: u32) -> Self {
        PlayerLocation::Room { room_id }
    }

    pub fn as_area_ref(self) -> Option<AreaRef> {
        match self {
            PlayerLocation::Area { zone_q, zone_r, area_id } =>
                Some(AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id }),
            PlayerLocation::Room { .. } => None,
        }
    }

    pub fn as_room_id(self) -> Option<u32> {
        match self {
            PlayerLocation::Room { room_id } => Some(room_id),
            PlayerLocation::Area { .. } => None,
        }
    }
}

impl HexCoord {
    pub const fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    /// Returns the six axial neighbors in direction order: N, NE, SE, S, SW, NW.
    /// Only hex directions (no E/W/Up/Down) — Area navigation only.
    pub fn neighbors(self) -> [(Direction, HexCoord); 6] {
        [
            (Direction::North,     HexCoord::new(self.q,     self.r - 1)),
            (Direction::NorthEast, HexCoord::new(self.q + 1, self.r - 1)),
            (Direction::SouthEast, HexCoord::new(self.q + 1, self.r)),
            (Direction::South,     HexCoord::new(self.q,     self.r + 1)),
            (Direction::SouthWest, HexCoord::new(self.q - 1, self.r + 1)),
            (Direction::NorthWest, HexCoord::new(self.q - 1, self.r)),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_coord_equality() {
        assert_eq!(HexCoord::new(0, 0), HexCoord::new(0, 0));
        assert_ne!(HexCoord::new(1, 0), HexCoord::new(0, 0));
    }

    #[test]
    fn hex_coord_hash_usable_as_map_key() {
        let mut map = std::collections::HashMap::new();
        map.insert(HexCoord::new(0, 0), "origin");
        map.insert(HexCoord::new(1, -1), "northeast");
        assert_eq!(map[&HexCoord::new(0, 0)], "origin");
        assert_eq!(map[&HexCoord::new(1, -1)], "northeast");
    }

    #[test]
    fn neighbors_returns_six() {
        let center = HexCoord::new(0, 0);
        let neighbors = center.neighbors();
        assert_eq!(neighbors.len(), 6);
    }

    #[test]
    fn north_neighbor_decrements_r() {
        let center = HexCoord::new(3, 4);
        let (dir, coord) = center.neighbors()[0];
        assert_eq!(dir, Direction::North);
        assert_eq!(coord, HexCoord::new(3, 3));
    }

    #[test]
    fn south_neighbor_increments_r() {
        let center = HexCoord::new(3, 4);
        let (dir, coord) = center.neighbors()[3];
        assert_eq!(dir, Direction::South);
        assert_eq!(coord, HexCoord::new(3, 5));
    }
}
