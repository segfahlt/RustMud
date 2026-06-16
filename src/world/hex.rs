use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::room::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
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
