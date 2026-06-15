use std::collections::HashMap;

use crate::commands::{help_text, Command};
use crate::mob::{MobCore, Player};
use crate::persist::CharacterSave;
use crate::world::{Direction, ObjectInstance, RoomRef, World};

pub struct GameState {
    pub world:   World,
    pub players: HashMap<u32, Player>,  // client_id → Player
}

impl GameState {
    pub fn new(world: World) -> Self {
        GameState { world, players: HashMap::new() }
    }

    pub fn add_player(&mut self, client_id: u32, character_id: &str, name: &str, location: RoomRef) {
        let core = MobCore::new(client_id, name, 100, location);
        self.players.insert(client_id, Player::new(core, character_id));
    }

    pub fn remove_player(&mut self, client_id: u32) {
        self.players.remove(&client_id);
    }

    pub fn snapshot_character(&self, client_id: u32) -> Option<CharacterSave> {
        let p = self.players.get(&client_id)?;
        Some(CharacterSave {
            zone_id:    p.core.location.zone_id,
            room_id:    p.core.location.room_id,
            health:     p.core.health,
            max_health: p.core.max_health,
            inventory:  p.inventory.clone(),
        })
    }
}

// Returns (output_text, keep_playing).
pub fn execute(cmd: Command, client_id: u32, state: &mut GameState) -> (String, bool) {
    match cmd {
        Command::Look(None)      => (describe_location(client_id, state), true),
        Command::Look(Some(dir)) => (look_direction(dir, client_id, state), true),
        Command::Examine(target) => (cmd_examine(&target, client_id, state), true),
        Command::Go(dir)         => (go_direction(dir, client_id, state), true),
        Command::Get(target)     => (cmd_get(&target, client_id, state), true),
        Command::Drop(target)    => (cmd_drop(&target, client_id, state), true),
        Command::Inventory       => (cmd_inventory(client_id, state), true),
        Command::WorldMap        => (state.world.world_map.render(), true),
        Command::Help(topic)     => (help_text(topic.as_deref()), true),
        Command::Quit            => ("Farewell.\n".to_string(), false),
        // Admin commands are intercepted in the connection layer before reaching here.
        Command::Shutdown | Command::Reboot | Command::RebootRefresh =>
            ("(admin command reached execute — this is a bug)\n".to_string(), true),
    }
}

pub fn describe_location(client_id: u32, state: &GameState) -> String {
    let player = match state.players.get(&client_id) {
        Some(p) => p,
        None    => return "(You don't exist. This is a bug.)\n".to_string(),
    };
    let loc = player.core.location;
    match state.world.get_room(loc.zone_id, loc.room_id) {
        Some(room) => room.render(&state.world.object_registry),
        None       => "(You are nowhere. This is a bug.)\n".to_string(),
    }
}

fn look_direction(dir: Direction, client_id: u32, state: &GameState) -> String {
    let player = match state.players.get(&client_id) {
        Some(p) => p,
        None    => return "(You don't exist. This is a bug.)\n".to_string(),
    };
    let loc = player.core.location;
    let room = match state.world.get_room(loc.zone_id, loc.room_id) {
        Some(r) => r,
        None    => return "(You are nowhere. This is a bug.)\n".to_string(),
    };
    match room.exits.get(&dir) {
        Some(dest) => match state.world.get_room(dest.zone_id, dest.room_id) {
            Some(dest_room) => format!("To the {}: {}\n", dir, dest_room.name),
            None            => String::new(),
        },
        None => format!("There is nothing to the {}.\n", dir),
    }
}

fn go_direction(dir: Direction, client_id: u32, state: &mut GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };
    // Copy dest out to end the borrow on state.world before mutating state.players.
    let dest = state.world
        .get_room(loc.zone_id, loc.room_id)
        .and_then(|room| room.exits.get(&dir).copied());

    match dest {
        Some(new_loc) => {
            state.players.get_mut(&client_id).unwrap().core.location = new_loc;
            describe_location(client_id, state)
        }
        None => "You can't go that way.\n".to_string(),
    }
}

fn cmd_examine(target: &str, client_id: u32, state: &GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };

    // Check fixtures in the current room first.
    if let Some(room) = state.world.get_room(loc.zone_id, loc.room_id) {
        if let Some(fixture) = room.fixtures.iter().find(|f| f.matches_name(target)) {
            return format!("{}\n", fixture.examine);
        }
        // Then objects on the floor.
        let registry = &state.world.object_registry;
        if let Some(obj) = room.objects.iter().find(|o| {
            registry.get(&o.template_id)
                .map(|t| t.matches_name(target))
                .unwrap_or(false)
        }) {
            return format!("{}\n", obj.description(registry));
        }
    }

    // Then inventory.
    let registry = &state.world.object_registry;
    if let Some(p) = state.players.get(&client_id) {
        if let Some(obj) = p.inventory.iter().find(|o| {
            registry.get(&o.template_id)
                .map(|t| t.matches_name(target))
                .unwrap_or(false)
        }) {
            return format!("{}\n", obj.description(registry));
        }
    }

    format!("You don't see any '{}' here.\n", target)
}

fn cmd_get(target: &str, client_id: u32, state: &mut GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };

    // Find the object and extract the info we need before releasing borrows.
    let result: Option<(usize, ObjectInstance, String)> = {
        let room = match state.world.get_room(loc.zone_id, loc.room_id) {
            Some(r) => r,
            None    => return "(You are nowhere.)\n".to_string(),
        };
        let registry = &state.world.object_registry;
        room.objects.iter().enumerate().find_map(|(idx, obj)| {
            registry.get(&obj.template_id).and_then(|tmpl| {
                if tmpl.matches_name(target) {
                    Some((idx, obj.clone(), tmpl.short.clone()))
                } else {
                    None
                }
            })
        })
    };

    match result {
        None => format!("You don't see any '{}' here.\n", target),
        Some((idx, obj, short)) => {
            if let Some(room) = state.world.get_room_mut(loc.zone_id, loc.room_id) {
                room.objects.remove(idx);
            }
            if let Some(p) = state.players.get_mut(&client_id) {
                p.inventory.push(obj);
            }
            format!("You pick up {}.\n", short)
        }
    }
}

fn cmd_drop(target: &str, client_id: u32, state: &mut GameState) -> String {
    let (loc, result) = {
        let player = match state.players.get(&client_id) {
            Some(p) => p,
            None    => return String::new(),
        };
        let registry = &state.world.object_registry;
        let result = player.inventory.iter().enumerate().find_map(|(idx, obj)| {
            registry.get(&obj.template_id).and_then(|tmpl| {
                if tmpl.matches_name(target) {
                    Some((idx, obj.clone(), tmpl.short.clone()))
                } else {
                    None
                }
            })
        });
        (player.core.location, result)
    };

    match result {
        None => format!("You aren't carrying any '{}'.\n", target),
        Some((idx, obj, short)) => {
            if let Some(p) = state.players.get_mut(&client_id) {
                p.inventory.remove(idx);
            }
            if let Some(room) = state.world.get_room_mut(loc.zone_id, loc.room_id) {
                room.objects.push(obj);
            }
            format!("You drop {}.\n", short)
        }
    }
}

fn cmd_inventory(client_id: u32, state: &GameState) -> String {
    let player = match state.players.get(&client_id) {
        Some(p) => p,
        None    => return String::new(),
    };
    if player.inventory.is_empty() {
        return "You are carrying nothing.\n".to_string();
    }
    let registry = &state.world.object_registry;
    let mut out = "You are carrying:\n".to_string();
    for obj in &player.inventory {
        out.push_str(&format!("  {} ({})\n", obj.short(registry), obj.condition.label()));
    }
    out
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Direction, Room, RoomRef, World, Zone};
    use std::collections::HashMap;

    const CLIENT: u32 = 0;
    const START:  RoomRef = RoomRef { zone_id: 1, room_id: 1 };

    fn make_state() -> GameState {
        let mut world = World::new();
        let mut zone = Zone::new(1, "Test Zone", "");
        zone.add_room(Room {
            id: 1,
            name: "Start Room".to_string(),
            description: "The starting room.".to_string(),
            exits: HashMap::from([
                (Direction::North, RoomRef { zone_id: 1, room_id: 2 }),
            ]),
            fixtures: vec![],
            objects: vec![],
        });
        zone.add_room(Room {
            id: 2,
            name: "North Room".to_string(),
            description: "North of start.".to_string(),
            exits: HashMap::from([
                (Direction::South, RoomRef { zone_id: 1, room_id: 1 }),
            ]),
            fixtures: vec![],
            objects: vec![],
        });
        world.add_zone(zone);
        let mut state = GameState::new(world);
        state.add_player(CLIENT, "tester", "Tester", START);
        state
    }

    #[test]
    fn go_north_moves_player() {
        let mut state = make_state();
        assert_eq!(state.players[&CLIENT].core.location.room_id, 1);
        let (_, cont) = execute(Command::Go(Direction::North), CLIENT, &mut state);
        assert!(cont);
        assert_eq!(state.players[&CLIENT].core.location.room_id, 2);
    }

    #[test]
    fn go_blocked_keeps_location() {
        let mut state = make_state();
        execute(Command::Go(Direction::East), CLIENT, &mut state);
        assert_eq!(state.players[&CLIENT].core.location.room_id, 1);
    }

    #[test]
    fn go_and_back_returns_to_start() {
        let mut state = make_state();
        execute(Command::Go(Direction::North), CLIENT, &mut state);
        execute(Command::Go(Direction::South), CLIENT, &mut state);
        assert_eq!(state.players[&CLIENT].core.location.room_id, 1);
    }

    #[test]
    fn quit_returns_false() {
        let mut state = make_state();
        let (_, cont) = execute(Command::Quit, CLIENT, &mut state);
        assert!(!cont);
    }

    #[test]
    fn look_returns_true() {
        let mut state = make_state();
        let (_, cont) = execute(Command::Look(None), CLIENT, &mut state);
        assert!(cont);
    }

    #[test]
    fn inventory_empty() {
        let mut state = make_state();
        let (out, _) = execute(Command::Inventory, CLIENT, &mut state);
        assert!(out.contains("nothing"));
    }

    #[test]
    fn multiple_clients_independent_locations() {
        let mut state = make_state();
        state.add_player(1, "tester2", "Tester2", START);
        execute(Command::Go(Direction::North), CLIENT, &mut state);
        assert_eq!(state.players[&CLIENT].core.location.room_id, 2);
        assert_eq!(state.players[&1].core.location.room_id, 1);
    }
}
