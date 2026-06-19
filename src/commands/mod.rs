pub mod registry;

use std::fmt;
use std::sync::OnceLock;

use crate::world::{Direction, PlayerLocation};
use registry::{Category, Registry};

// --- OHelpQuery ---

pub enum OHelpQuery {
    Overview,
    List,
    Search(String),  // exact id → full detail; otherwise name substring → list
    Desc(String),    // description substring → list
}

// --- Command ---
// The parsed result of a player's input.
// Variants carry typed arguments — no raw strings past the parser.

pub enum Command {
    Look(Option<Direction>),    // look | look <dir>
    Examine(String),            // look at <thing> | examine <thing>
    Go(Direction),
    Enter(Direction),           // enter <dir> — enter a fixture when area exit also exists
    Get(String),                // get <thing>
    Drop(String),               // drop <thing>
    Read(String),               // read <thing>
    Eat(String),                // eat <food>
    Drink(String),              // drink <liquid>
    UseItem(String),            // use <item>   — generic consume trigger
    PutIn { item: String, container: String },     // put <item> in <container>
    GetFrom { item: String, container: String },   // get <item> from <container>
    LookIn(String),             // look in <container>
    Wield(String),              // wield <weapon>
    Wear(String),               // wear <armor>
    Remove(String),             // remove <item>  — unequip
    Equipment,                  // equipment | eq
    Inventory,                  // inventory | i
    WorldMap,                   // wmap | worldmap
    Help(Option<String>),       // help | help <topic>
    OHelp(OHelpQuery),          // ohelp object reference system
    Quit,
    Shutdown,                   // kill game + gateway  [Admin]
    Reboot,                     // graceful game restart [Admin|Dev]
    RebootRefresh,              // game restart + player reset [Admin]
    Teleport(PlayerLocation),   // teleport to room/area  [Admin]
}

// --- ParseError ---

#[derive(Debug)]
pub enum ParseError {
    Empty,
    UnknownCommand(String),
    AmbiguousCommand(String),
    UnknownDirection(String),
    MissingDirection,
    MissingTarget(String),      // command that requires a target but got none
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Empty =>
                write!(f, "No command entered."),
            ParseError::UnknownCommand(cmd) =>
                write!(f, "Unknown command: '{}'. Type 'help' for a list.", cmd),
            ParseError::AmbiguousCommand(cmd) =>
                write!(f, "Ambiguous command: '{}' — be more specific.", cmd),
            ParseError::UnknownDirection(dir) =>
                write!(f, "Unknown direction: '{}'.", dir),
            ParseError::MissingDirection =>
                write!(f, "'go' requires a direction: n, ne, nw, s, se, sw, e, w, up, down."),
            ParseError::MissingTarget(cmd) =>
                write!(f, "{}", cmd),
        }
    }
}

// --- Registry (initialized once, lives for the program's lifetime) ---

static REGISTRY: OnceLock<Registry> = OnceLock::new();

fn registry() -> &'static Registry {
    REGISTRY.get_or_init(Registry::build)
}

// --- parse ---
// Splits input into verb + rest, resolves the verb via the registry,
// then delegates argument parsing to the matched CommandDef.

pub fn parse(input: &str) -> Result<Command, ParseError> {
    let lowered = input.trim().to_lowercase();

    if lowered.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut parts = lowered.splitn(2, ' ');
    let verb = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim();

    let def = registry().find(verb)?;
    (def.parse)(rest)
}

// --- help_text ---
// Returns formatted help, either a full listing grouped by category,
// or detailed info for a single command.

pub fn help_text(topic: Option<&str>) -> String {
    let reg = registry();

    match topic {
        Some(t) => match reg.find(t) {
            Ok(def) => {
                let aliases = if def.aliases.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", def.aliases.join(", "))
                };
                format!(
                    "{}{} [{}]\n  Usage:  {}\n  {}",
                    def.name, aliases, def.category.label(), def.usage, def.description
                )
            }
            Err(_) => format!("No help available for '{}'.", t),
        },

        None => {
            let order = [
                Category::Movement,
                Category::Items,
                Category::Info,
                Category::Communication,
                Category::Admin,
            ];
            let mut out = String::from("Available commands:\n");
            for cat in &order {
                let cmds: Vec<_> = reg.all().iter().filter(|d| &d.category == cat).collect();
                if cmds.is_empty() { continue; }
                out.push('\n');
                out.push_str(cat.label());
                out.push('\n');
                for def in cmds {
                    let col = if def.aliases.is_empty() {
                        def.name.to_string()
                    } else {
                        format!("{} ({})", def.name, def.aliases.join(", "))
                    };
                    out.push_str(&format!("  {:<20}{}\n", col, def.description));
                }
            }
            out
        }
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    // --- look ---
    #[test] fn parse_look()           { assert!(matches!(parse("look"),    Ok(Command::Look(None)))); }
    #[test] fn parse_look_short()     { assert!(matches!(parse("l"),       Ok(Command::Look(None)))); }
    #[test] fn parse_look_prefix()    { assert!(matches!(parse("lo"),      Ok(Command::Look(None)))); }
    #[test] fn parse_look_north()     { assert!(matches!(parse("look north"),    Ok(Command::Look(Some(Direction::North))))); }
    #[test] fn parse_look_dir_short() { assert!(matches!(parse("look n"),        Ok(Command::Look(Some(Direction::North))))); }
    #[test] fn parse_look_ne()        { assert!(matches!(parse("look ne"),       Ok(Command::Look(Some(Direction::NorthEast))))); }
    #[test] fn parse_look_northeast() { assert!(matches!(parse("look northeast"),Ok(Command::Look(Some(Direction::NorthEast))))); }

    // --- examine (via look at) ---
    #[test] fn parse_look_at_thing()  { assert!(matches!(parse("look at forge"), Ok(Command::Examine(_)))); }
    #[test] fn parse_look_target()    { assert!(matches!(parse("look forge"),    Ok(Command::Examine(_)))); }
    #[test] fn parse_examine()        { assert!(matches!(parse("examine forge"), Ok(Command::Examine(_)))); }
    #[test] fn parse_examine_prefix() { assert!(matches!(parse("ex forge"),      Ok(Command::Examine(_)))); }

    // --- inventory ---
    #[test] fn parse_inventory()      { assert!(matches!(parse("inventory"), Ok(Command::Inventory))); }
    #[test] fn parse_inventory_alias(){ assert!(matches!(parse("i"),         Ok(Command::Inventory))); }
    #[test] fn parse_inventory_inv()  { assert!(matches!(parse("inv"),       Ok(Command::Inventory))); }

    // --- get / drop / read ---
    #[test] fn parse_get()            { assert!(matches!(parse("get knife"), Ok(Command::Get(_)))); }
    #[test] fn parse_get_take()       { assert!(matches!(parse("take knife"),Ok(Command::Get(_)))); }
    #[test] fn parse_drop()           { assert!(matches!(parse("drop knife"),Ok(Command::Drop(_)))); }
    #[test] fn parse_get_nothing()    { assert!(matches!(parse("get"),       Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_drop_nothing()   { assert!(matches!(parse("drop"),      Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_read()           { assert!(matches!(parse("read note"),   Ok(Command::Read(_)))); }
    #[test] fn parse_read_nothing()   { assert!(matches!(parse("read"),        Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_put_in()         { assert!(matches!(parse("put knife in bag"), Ok(Command::PutIn { .. }))); }
    #[test] fn parse_put_in_nothing() { assert!(matches!(parse("put knife"),        Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_get_from()       { assert!(matches!(parse("get knife from bag"), Ok(Command::GetFrom { .. }))); }
    #[test] fn parse_look_in()        { assert!(matches!(parse("look in bag"),    Ok(Command::LookIn(_)))); }
    #[test] fn parse_eat()            { assert!(matches!(parse("eat ration"),  Ok(Command::Eat(_)))); }
    #[test] fn parse_eat_nothing()    { assert!(matches!(parse("eat"),         Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_drink()          { assert!(matches!(parse("drink water"), Ok(Command::Drink(_)))); }
    #[test] fn parse_drink_nothing()  { assert!(matches!(parse("drink"),       Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_use_item()       { assert!(matches!(parse("use stimpak"), Ok(Command::UseItem(_)))); }
    #[test] fn parse_use_nothing()    { assert!(matches!(parse("use"),         Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_wield()          { assert!(matches!(parse("wield knife"), Ok(Command::Wield(_)))); }
    #[test] fn parse_wield_nothing()  { assert!(matches!(parse("wield"),       Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_wear()           { assert!(matches!(parse("wear vest"),   Ok(Command::Wear(_)))); }
    #[test] fn parse_wear_nothing()   { assert!(matches!(parse("wear"),        Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_remove()         { assert!(matches!(parse("remove vest"), Ok(Command::Remove(_)))); }
    #[test] fn parse_remove_nothing() { assert!(matches!(parse("remove"),      Err(ParseError::MissingTarget(_)))); }
    #[test] fn parse_equipment()      { assert!(matches!(parse("equipment"),   Ok(Command::Equipment))); }
    #[test] fn parse_eq_alias()       { assert!(matches!(parse("eq"),          Ok(Command::Equipment))); }

    // --- go ---
    #[test] fn parse_go_north()     { assert!(matches!(parse("go north"),     Ok(Command::Go(Direction::North)))); }
    #[test] fn parse_go_south()     { assert!(matches!(parse("go south"),     Ok(Command::Go(Direction::South)))); }
    #[test] fn parse_go_east()      { assert!(matches!(parse("go east"),      Ok(Command::Go(Direction::East)))); }
    #[test] fn parse_go_west()      { assert!(matches!(parse("go west"),      Ok(Command::Go(Direction::West)))); }
    #[test] fn parse_go_up()        { assert!(matches!(parse("go up"),        Ok(Command::Go(Direction::Up)))); }
    #[test] fn parse_go_down()      { assert!(matches!(parse("go down"),      Ok(Command::Go(Direction::Down)))); }
    #[test] fn parse_go_northeast() { assert!(matches!(parse("go northeast"), Ok(Command::Go(Direction::NorthEast)))); }
    #[test] fn parse_go_northwest() { assert!(matches!(parse("go northwest"), Ok(Command::Go(Direction::NorthWest)))); }
    #[test] fn parse_go_southeast() { assert!(matches!(parse("go southeast"), Ok(Command::Go(Direction::SouthEast)))); }
    #[test] fn parse_go_southwest() { assert!(matches!(parse("go southwest"), Ok(Command::Go(Direction::SouthWest)))); }
    #[test] fn parse_go_short()     { assert!(matches!(parse("go n"),         Ok(Command::Go(Direction::North)))); }
    #[test] fn parse_go_ne_short()  { assert!(matches!(parse("go ne"),        Ok(Command::Go(Direction::NorthEast)))); }

    // --- bare directions ---
    #[test] fn parse_bare_north()     { assert!(matches!(parse("north"),     Ok(Command::Go(Direction::North)))); }
    #[test] fn parse_bare_northeast() { assert!(matches!(parse("northeast"), Ok(Command::Go(Direction::NorthEast)))); }
    #[test] fn parse_alias_n()        { assert!(matches!(parse("n"),         Ok(Command::Go(Direction::North)))); }
    #[test] fn parse_alias_s()        { assert!(matches!(parse("s"),         Ok(Command::Go(Direction::South)))); }
    #[test] fn parse_alias_e()        { assert!(matches!(parse("e"),         Ok(Command::Go(Direction::East)))); }
    #[test] fn parse_alias_w()        { assert!(matches!(parse("w"),         Ok(Command::Go(Direction::West)))); }
    #[test] fn parse_alias_ne()       { assert!(matches!(parse("ne"),        Ok(Command::Go(Direction::NorthEast)))); }
    #[test] fn parse_alias_nw()       { assert!(matches!(parse("nw"),        Ok(Command::Go(Direction::NorthWest)))); }
    #[test] fn parse_alias_se()       { assert!(matches!(parse("se"),        Ok(Command::Go(Direction::SouthEast)))); }
    #[test] fn parse_alias_sw()       { assert!(matches!(parse("sw"),        Ok(Command::Go(Direction::SouthWest)))); }

    // --- teleport ---
    #[test]
    fn parse_teleport_room() {
        assert!(matches!(parse("teleport 1"), Ok(Command::Teleport(PlayerLocation::Room { room_id: 1 }))));
    }
    #[test]
    fn parse_teleport_area() {
        assert!(matches!(
            parse("teleport 0 1 5"),
            Ok(Command::Teleport(PlayerLocation::Area { zone_q: 0, zone_r: 1, area_id: 5 }))
        ));
    }
    #[test]
    fn parse_goto_alias() {
        assert!(matches!(parse("goto 2"), Ok(Command::Teleport(PlayerLocation::Room { room_id: 2 }))));
    }
    #[test]
    fn parse_teleport_no_args()  { assert!(matches!(parse("teleport"),     Err(ParseError::MissingTarget(_)))); }
    #[test]
    fn parse_teleport_bad_id()   { assert!(matches!(parse("teleport abc"), Err(ParseError::MissingTarget(_)))); }
    #[test]
    fn parse_teleport_two_args() { assert!(matches!(parse("teleport 0 1"), Err(ParseError::MissingTarget(_)))); }

    // --- help ---
    #[test] fn parse_help()       { assert!(matches!(parse("help"),      Ok(Command::Help(None)))); }
    #[test] fn parse_help_topic() { assert!(matches!(parse("help look"), Ok(Command::Help(Some(_))))); }
    #[test] fn parse_help_short() { assert!(matches!(parse("h"),         Ok(Command::Help(None)))); }

    // --- quit ---
    #[test] fn parse_quit()      { assert!(matches!(parse("quit"), Ok(Command::Quit))); }
    #[test] fn parse_quit_short(){ assert!(matches!(parse("q"),    Ok(Command::Quit))); }
    #[test] fn parse_quit_exit() { assert!(matches!(parse("exit"), Ok(Command::Quit))); }

    // --- whitespace and case ---
    #[test] fn parse_trims_whitespace() { assert!(matches!(parse("  look  "), Ok(Command::Look(None)))); }
    #[test] fn parse_case_insensitive() { assert!(matches!(parse("GO NORTH"), Ok(Command::Go(Direction::North)))); }
    #[test] fn parse_mixed_case()       { assert!(matches!(parse("Look"),     Ok(Command::Look(None)))); }

    // --- errors ---
    #[test] fn parse_empty()           { assert!(matches!(parse(""),           Err(ParseError::Empty))); }
    #[test] fn parse_whitespace_only() { assert!(matches!(parse("   "),        Err(ParseError::Empty))); }
    #[test] fn parse_go_alone()        { assert!(matches!(parse("go"),         Err(ParseError::MissingDirection))); }
    #[test] fn parse_go_bad_dir()      { assert!(matches!(parse("go sideways"),Err(ParseError::UnknownDirection(_)))); }
    #[test] fn parse_unknown_command() { assert!(matches!(parse("fly"),        Err(ParseError::UnknownCommand(_)))); }
}
