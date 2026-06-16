use std::collections::HashMap;

use crate::commands::{help_text, Command};
use crate::mob::{MobCore, Player};
use crate::persist::CharacterSave;
use crate::world::{Area, AreaRef, Direction, ExitDestination, HexCoord, ObjectInstance, PlayerLocation, World};

pub struct GameState {
    pub world:   World,
    pub players: HashMap<u32, Player>,  // client_id → Player
}

impl GameState {
    pub fn new(world: World) -> Self {
        GameState { world, players: HashMap::new() }
    }

    pub fn add_player(
        &mut self,
        client_id:    u32,
        character_id: &str,
        name:         &str,
        location:     PlayerLocation,
    ) {
        let core = MobCore::new(client_id, name, 100, location);
        self.players.insert(client_id, Player::new(core, character_id));
    }

    pub fn remove_player(&mut self, client_id: u32) {
        self.players.remove(&client_id);
    }

    pub fn snapshot_character(&self, client_id: u32) -> Option<CharacterSave> {
        let p = self.players.get(&client_id)?;
        Some(CharacterSave {
            location:   p.core.location,
            health:     p.core.health,
            max_health: p.core.max_health,
            inventory:  p.inventory.clone(),
            last_area:  p.last_area,
        })
    }
}

/// Area mode (outdoor): only hex directions. Room mode: all 10 directions.
fn is_valid_direction(dir: Direction, in_area: bool) -> bool {
    if !in_area {
        return true; // Rooms allow all 10 directions
    }
    matches!(dir,
        Direction::North | Direction::South |
        Direction::NorthEast | Direction::NorthWest |
        Direction::SouthEast | Direction::SouthWest
    )
}

// Returns (output_text, keep_playing).
pub fn execute(cmd: Command, client_id: u32, state: &mut GameState) -> (String, bool) {
    match cmd {
        Command::Look(None)      => (describe_location(client_id, state), true),
        Command::Look(Some(dir)) => (look_direction(dir, client_id, state), true),
        Command::Examine(target) => (cmd_examine(&target, client_id, state), true),
        Command::Go(dir)         => (go_direction(dir, client_id, state), true),
        Command::Enter(dir)      => (enter_fixture(dir, client_id, state), true),
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
    match player.core.location {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
            match state.world.get_area(area_ref) {
                Some(area) => {
                    let zone_name = state.world.get_zone_name(area_ref.zone).unwrap_or("Unknown");
                    render_area(area, zone_name, &state.world.object_registry)
                }
                None => "(You are nowhere. This is a bug.)\n".to_string(),
            }
        }
        PlayerLocation::Room { room_id } => {
            match state.world.get_room(room_id) {
                Some(room) => room.render(&state.world.object_registry),
                None       => "(You are in an unregistered room. This is a bug.)\n".to_string(),
            }
        }
    }
}

fn render_area(area: &Area, zone_name: &str, registry: &crate::world::ObjectRegistry) -> String {
    let exits = if area.exits.is_empty() {
        "none".to_string()
    } else {
        let mut dirs: Vec<String> = area.exits.keys().map(|d| d.to_string()).collect();
        dirs.sort();
        dirs.join(", ")
    };

    let header = format!("[ {} > {} ]", zone_name, area.name);
    let mut out = format!("{}\n{}", header, area.description);

    let mut extras = Vec::new();
    for fixture in &area.fixtures {
        let line = fixture.state_line();
        if !line.is_empty() {
            extras.push(line.to_string());
        }
    }
    for obj in &area.objects {
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

fn look_direction(dir: Direction, client_id: u32, state: &GameState) -> String {
    let player = match state.players.get(&client_id) {
        Some(p) => p,
        None    => return "(You don't exist. This is a bug.)\n".to_string(),
    };
    match player.core.location {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
            let area = match state.world.get_area(area_ref) {
                Some(a) => a,
                None    => return "(You are nowhere. This is a bug.)\n".to_string(),
            };
            match area.exits.get(&dir) {
                Some(dest) => match state.world.get_area(*dest) {
                    Some(dest_area) => format!("To the {}: {}\n", dir, dest_area.name),
                    None            => String::new(),
                },
                None => format!("There is nothing to the {}.\n", dir),
            }
        }
        PlayerLocation::Room { room_id } => {
            let room = match state.world.get_room(room_id) {
                Some(r) => r,
                None    => return "(You are nowhere. This is a bug.)\n".to_string(),
            };
            match room.exits.get(&dir) {
                Some(ExitDestination::Room { room_id: dest_id }) => {
                    match state.world.get_room(*dest_id) {
                        Some(dest_room) => format!("To the {}: {}\n", dir, dest_room.name),
                        None            => String::new(),
                    }
                }
                Some(ExitDestination::Fixture(_)) => {
                    format!("A building entrance leads {}.\n", dir)
                }
                None => format!("There is nothing to the {}.\n", dir),
            }
        }
    }
}

fn go_direction(dir: Direction, client_id: u32, state: &mut GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };

    match loc {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            if !is_valid_direction(dir, true) {
                return "You can't go that way.\n".to_string();
            }
            let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };

            // Area exit takes priority.
            let area_exit = state.world
                .get_area(area_ref)
                .and_then(|area| area.exits.get(&dir).copied());
            if let Some(new_ref) = area_exit {
                state.players.get_mut(&client_id).unwrap().core.location =
                    PlayerLocation::area(new_ref.zone, new_ref.area_id);
                return describe_location(client_id, state);
            }

            // No area exit — auto-enter any gateway fixture in this area.
            let gateway_room = state.world
                .get_area(area_ref)
                .and_then(|area| area.fixtures.iter().find_map(|f| f.connects_to_room));
            if let Some(room_id) = gateway_room {
                let p = state.players.get_mut(&client_id).unwrap();
                p.last_area = Some(loc);
                p.core.location = PlayerLocation::room(room_id);
                return describe_location(client_id, state);
            }

            "You can't go that way.\n".to_string()
        }
        PlayerLocation::Room { room_id } => {
            let dest = state.world
                .get_room(room_id)
                .and_then(|room| room.exits.get(&dir).cloned());
            match dest {
                Some(ExitDestination::Room { room_id: new_id }) => {
                    state.players.get_mut(&client_id).unwrap().core.location =
                        PlayerLocation::room(new_id);
                    describe_location(client_id, state)
                }
                Some(ExitDestination::Fixture(fixture_ref)) => {
                    let p = state.players.get_mut(&client_id).unwrap();
                    let return_loc = p.last_area.take().unwrap_or_else(|| {
                        PlayerLocation::area(fixture_ref.zone, fixture_ref.area_id)
                    });
                    p.core.location = return_loc;
                    describe_location(client_id, state)
                }
                None => "You can't go that way.\n".to_string(),
            }
        }
    }
}

fn enter_fixture(dir: Direction, client_id: u32, state: &mut GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };

    let area_ref = match loc.as_area_ref() {
        Some(r) => r,
        None    => return "You are already inside.\n".to_string(),
    };

    if !is_valid_direction(dir, true) {
        return "You can't go that way.\n".to_string();
    }

    // Find a gateway fixture (explicit enter, even when an area exit also exists in this dir).
    let gateway_room = state.world
        .get_area(area_ref)
        .and_then(|area| area.fixtures.iter().find_map(|f| f.connects_to_room));

    match gateway_room {
        Some(room_id) => {
            let p = state.players.get_mut(&client_id).unwrap();
            p.last_area = Some(loc);
            p.core.location = PlayerLocation::room(room_id);
            describe_location(client_id, state)
        }
        None => "There's nothing to enter here.\n".to_string(),
    }
}

fn cmd_examine(target: &str, client_id: u32, state: &GameState) -> String {
    let loc = match state.players.get(&client_id) {
        Some(p) => p.core.location,
        None    => return String::new(),
    };

    let (fixtures, objects) = match loc {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
            match state.world.get_area(area_ref) {
                Some(a) => (a.fixtures.as_slice(), a.objects.as_slice()),
                None    => return "(You are nowhere.)\n".to_string(),
            }
        }
        PlayerLocation::Room { room_id } => {
            match state.world.get_room(room_id) {
                Some(r) => (r.fixtures.as_slice(), r.objects.as_slice()),
                None    => return "(You are nowhere.)\n".to_string(),
            }
        }
    };

    if let Some(fixture) = fixtures.iter().find(|f| f.matches_name(target)) {
        return format!("{}\n", fixture.examine);
    }

    let registry = &state.world.object_registry;
    if let Some(obj) = objects.iter().find(|o| {
        registry.get(&o.template_id).map(|t| t.matches_name(target)).unwrap_or(false)
    }) {
        return format!("{}\n", obj.description(registry));
    }

    if let Some(p) = state.players.get(&client_id) {
        if let Some(obj) = p.inventory.iter().find(|o| {
            registry.get(&o.template_id).map(|t| t.matches_name(target)).unwrap_or(false)
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

    let result: Option<(usize, ObjectInstance, String)> = match loc {
        PlayerLocation::Area { zone_q, zone_r, area_id } => {
            let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
            let area = match state.world.get_area(area_ref) {
                Some(a) => a,
                None    => return "(You are nowhere.)\n".to_string(),
            };
            let registry = &state.world.object_registry;
            area.objects.iter().enumerate().find_map(|(idx, obj)| {
                registry.get(&obj.template_id).and_then(|tmpl| {
                    if tmpl.matches_name(target) { Some((idx, obj.clone(), tmpl.short.clone())) } else { None }
                })
            })
        }
        PlayerLocation::Room { room_id } => {
            let room = match state.world.get_room(room_id) {
                Some(r) => r,
                None    => return "(You are nowhere.)\n".to_string(),
            };
            let registry = &state.world.object_registry;
            room.objects.iter().enumerate().find_map(|(idx, obj)| {
                registry.get(&obj.template_id).and_then(|tmpl| {
                    if tmpl.matches_name(target) { Some((idx, obj.clone(), tmpl.short.clone())) } else { None }
                })
            })
        }
    };

    match result {
        None => format!("You don't see any '{}' here.\n", target),
        Some((idx, obj, short)) => {
            match loc {
                PlayerLocation::Area { zone_q, zone_r, area_id } => {
                    let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
                    if let Some(area) = state.world.get_area_mut(area_ref) {
                        area.objects.remove(idx);
                    }
                }
                PlayerLocation::Room { room_id } => {
                    if let Some(room) = state.world.get_room_mut(room_id) {
                        room.objects.remove(idx);
                    }
                }
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
                if tmpl.matches_name(target) { Some((idx, obj.clone(), tmpl.short.clone())) } else { None }
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
            match loc {
                PlayerLocation::Area { zone_q, zone_r, area_id } => {
                    let area_ref = AreaRef { zone: HexCoord::new(zone_q, zone_r), area_id };
                    if let Some(area) = state.world.get_area_mut(area_ref) {
                        area.objects.push(obj);
                    }
                }
                PlayerLocation::Room { room_id } => {
                    if let Some(room) = state.world.get_room_mut(room_id) {
                        room.objects.push(obj);
                    }
                }
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
    use crate::world::{Area, AreaRef, HexCoord, World, Zone};
    use std::collections::HashMap;

    const CLIENT: u32 = 0;

    fn start_loc() -> PlayerLocation {
        PlayerLocation::area(HexCoord::new(0, 0), 1)
    }

    fn make_state() -> GameState {
        let mut world = World::new();
        let mut zone = Zone::new(HexCoord::new(0, 0), "Test Zone", "");
        zone.add_area(Area {
            id: 1,
            name: "Start Area".to_string(),
            description: "The starting area.".to_string(),
            exits: HashMap::from([
                (Direction::North, AreaRef { zone: HexCoord::new(0, 0), area_id: 2 }),
            ]),
            ..Area::default()
        });
        zone.add_area(Area {
            id: 2,
            name: "North Area".to_string(),
            description: "North of start.".to_string(),
            exits: HashMap::from([
                (Direction::South, AreaRef { zone: HexCoord::new(0, 0), area_id: 1 }),
            ]),
            ..Area::default()
        });
        world.add_zone(zone);
        let mut state = GameState::new(world);
        state.add_player(CLIENT, "tester", "Tester", start_loc());
        state
    }

    #[test]
    fn go_north_moves_player() {
        let mut state = make_state();
        assert_eq!(state.players[&CLIENT].core.location, start_loc());
        let (_, cont) = execute(Command::Go(Direction::North), CLIENT, &mut state);
        assert!(cont);
        let expected = PlayerLocation::area(HexCoord::new(0, 0), 2);
        assert_eq!(state.players[&CLIENT].core.location, expected);
    }

    #[test]
    fn go_blocked_keeps_location() {
        let mut state = make_state();
        execute(Command::Go(Direction::East), CLIENT, &mut state);
        assert_eq!(state.players[&CLIENT].core.location, start_loc());
    }

    #[test]
    fn go_and_back_returns_to_start() {
        let mut state = make_state();
        execute(Command::Go(Direction::North), CLIENT, &mut state);
        execute(Command::Go(Direction::South), CLIENT, &mut state);
        assert_eq!(state.players[&CLIENT].core.location, start_loc());
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
        state.add_player(1, "tester2", "Tester2", start_loc());
        execute(Command::Go(Direction::North), CLIENT, &mut state);
        let expected_north = PlayerLocation::area(HexCoord::new(0, 0), 2);
        assert_eq!(state.players[&CLIENT].core.location, expected_north);
        assert_eq!(state.players[&1].core.location, start_loc());
    }
}
