use std::collections::HashMap;

use super::area::Area;
use super::hex::HexCoord;

#[derive(Debug)]
pub struct Zone {
    pub coord:        HexCoord,
    pub name:         String,
    pub description:  String,
    pub biome_origin: String,
    pub coherence:    u8,
    pub radius_steps: u8,
    areas:            HashMap<u32, Area>,
}

impl Zone {
    pub fn new(coord: HexCoord, name: impl Into<String>, description: impl Into<String>) -> Self {
        Zone {
            coord,
            name: name.into(),
            description: description.into(),
            biome_origin: String::new(),
            coherence: 50,
            radius_steps: 1,
            areas: HashMap::new(),
        }
    }

    pub fn add_area(&mut self, area: Area) {
        self.areas.insert(area.id, area);
    }

    pub fn get_area(&self, area_id: u32) -> Option<&Area> {
        self.areas.get(&area_id)
    }

    pub fn get_area_mut(&mut self, area_id: u32) -> Option<&mut Area> {
        self.areas.get_mut(&area_id)
    }

    pub fn area_ids(&self) -> Vec<u32> {
        let mut ids: Vec<u32> = self.areas.keys().copied().collect();
        ids.sort();
        ids
    }

    pub fn areas(&self) -> impl Iterator<Item = &Area> {
        self.areas.values()
    }

    pub fn areas_mut(&mut self) -> impl Iterator<Item = &mut Area> {
        self.areas.values_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_area(id: u32) -> Area {
        Area {
            id,
            name: format!("Area {id}"),
            ..Area::default()
        }
    }

    fn coord() -> HexCoord { HexCoord::new(0, 0) }

    #[test]
    fn add_and_get_area() {
        let mut zone = Zone::new(coord(), "Test", "");
        zone.add_area(make_area(1));
        assert!(zone.get_area(1).is_some());
    }

    #[test]
    fn get_missing_area_returns_none() {
        assert!(Zone::new(coord(), "Test", "").get_area(99).is_none());
    }

    #[test]
    fn area_ids_are_sorted() {
        let mut zone = Zone::new(coord(), "Test", "");
        zone.add_area(make_area(3));
        zone.add_area(make_area(1));
        zone.add_area(make_area(2));
        assert_eq!(zone.area_ids(), vec![1, 2, 3]);
    }
}
