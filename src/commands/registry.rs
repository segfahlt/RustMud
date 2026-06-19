use crate::world::{Direction, HexCoord, PlayerLocation};
use super::{Command, OHelpQuery, ParseError};

// --- Category ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Movement,
    Items,
    Info,
    Communication,
    Admin,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Movement      => "Movement",
            Category::Items         => "Items",
            Category::Info          => "Info",
            Category::Communication => "Communication",
            Category::Admin         => "Admin",
        }
    }
}

// --- CommandDef ---

pub struct CommandDef {
    pub name:        &'static str,
    // Lower number = higher priority. Resolves which command wins when
    // multiple command names share a prefix (e.g. "l" with Look=1, Learn=2 → Look wins).
    pub priority:    u32,
    // Aliases are for shortcuts that cannot be expressed as name prefixes:
    // special characters ("?"), or full alternative words ("exit", "move").
    // Do NOT use aliases as abbreviations — prefix matching + priority handles that.
    pub aliases:     &'static [&'static str],
    pub category:    Category,
    pub usage:       &'static str,
    pub description: &'static str,
    pub parse:       fn(&str) -> Result<Command, ParseError>,
}

// --- Registry ---

pub struct Registry(Vec<CommandDef>);

impl Registry {
    pub fn build() -> Self {
        Registry(vec![
            CommandDef {
                name: "look", priority: 10, aliases: &[],
                category: Category::Info,
                usage: "look [direction | at <thing>]",
                description: "Look around, peek in a direction, or look at something.",
                parse: parse_look,
            },
            CommandDef {
                name: "examine", priority: 10, aliases: &[],
                category: Category::Items,
                usage: "examine <thing>",
                description: "Examine something closely.",
                parse: parse_examine,
            },
            CommandDef {
                name: "get", priority: 15, aliases: &["take"],
                category: Category::Items,
                usage: "get <thing>",
                description: "Pick up an object from the room.",
                parse: parse_get,
            },
            CommandDef {
                name: "drop", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "drop <thing>",
                description: "Drop something from your inventory.",
                parse: parse_drop,
            },
            CommandDef {
                name: "read", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "read <thing>",
                description: "Read a note, book, or data item.",
                parse: parse_read,
            },
            CommandDef {
                name: "eat", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "eat <food>",
                description: "Eat a food item.",
                parse: parse_eat,
            },
            CommandDef {
                name: "drink", priority: 20, aliases: &[],
                category: Category::Items,
                usage: "drink <liquid>",
                description: "Drink something.",
                parse: parse_drink,
            },
            CommandDef {
                name: "use", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "use <item>",
                description: "Use a consumable item.",
                parse: parse_use_item,
            },
            CommandDef {
                name: "wield", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "wield <weapon>",
                description: "Wield a weapon in your main hand.",
                parse: parse_wield,
            },
            CommandDef {
                name: "wear", priority: 15, aliases: &[],
                category: Category::Items,
                usage: "wear <armor>",
                description: "Put on a piece of armor.",
                parse: parse_wear,
            },
            CommandDef {
                name: "remove", priority: 15, aliases: &["unwield", "unequip"],
                category: Category::Items,
                usage: "remove <item>",
                description: "Remove a worn or wielded item.",
                parse: parse_remove,
            },
            CommandDef {
                name: "equipment", priority: 10, aliases: &["eq"],
                category: Category::Items,
                usage: "equipment",
                description: "Show what you have equipped.",
                parse: |_| Ok(Command::Equipment),
            },
            CommandDef {
                name: "inventory", priority: 10, aliases: &["i", "inv"],
                category: Category::Items,
                usage: "inventory",
                description: "List what you are carrying.",
                parse: |_| Ok(Command::Inventory),
            },
            CommandDef {
                name: "wmap", priority: 10, aliases: &["worldmap"],
                category: Category::Info,
                usage: "wmap",
                description: "Display the world geological survey map.",
                parse: |_| Ok(Command::WorldMap),
            },
            CommandDef {
                name: "go", priority: 10, aliases: &["move"],
                category: Category::Movement,
                usage: "go <direction>",
                description: "Move in a direction.",
                parse: parse_go,
            },
            CommandDef {
                name: "enter", priority: 10, aliases: &[],
                category: Category::Movement,
                usage: "enter <direction>",
                description: "Enter a building fixture when a regular exit also exists.",
                parse: parse_enter,
            },
            CommandDef {
                name: "north", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "north",
                description: "Move north.",
                parse: |_| Ok(Command::Go(Direction::North)),
            },
            CommandDef {
                name: "south", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "south",
                description: "Move south.",
                parse: |_| Ok(Command::Go(Direction::South)),
            },
            CommandDef {
                name: "east", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "east",
                description: "Move east.",
                parse: |_| Ok(Command::Go(Direction::East)),
            },
            CommandDef {
                name: "west", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "west",
                description: "Move west.",
                parse: |_| Ok(Command::Go(Direction::West)),
            },
            CommandDef {
                name: "up", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "up",
                description: "Move up.",
                parse: |_| Ok(Command::Go(Direction::Up)),
            },
            CommandDef {
                name: "down", priority: 5, aliases: &[],
                category: Category::Movement,
                usage: "down",
                description: "Move down.",
                parse: |_| Ok(Command::Go(Direction::Down)),
            },
            CommandDef {
                name: "northeast", priority: 6, aliases: &["ne"],
                category: Category::Movement,
                usage: "northeast",
                description: "Move northeast.",
                parse: |_| Ok(Command::Go(Direction::NorthEast)),
            },
            CommandDef {
                name: "northwest", priority: 6, aliases: &["nw"],
                category: Category::Movement,
                usage: "northwest",
                description: "Move northwest.",
                parse: |_| Ok(Command::Go(Direction::NorthWest)),
            },
            CommandDef {
                name: "southeast", priority: 6, aliases: &["se"],
                category: Category::Movement,
                usage: "southeast",
                description: "Move southeast.",
                parse: |_| Ok(Command::Go(Direction::SouthEast)),
            },
            CommandDef {
                name: "southwest", priority: 6, aliases: &["sw"],
                category: Category::Movement,
                usage: "southwest",
                description: "Move southwest.",
                parse: |_| Ok(Command::Go(Direction::SouthWest)),
            },
            CommandDef {
                name: "help", priority: 10, aliases: &["?"],
                category: Category::Info,
                usage: "help [command]",
                description: "Show available commands, or help for a specific command.",
                parse: parse_help,
            },
            CommandDef {
                name: "ohelp", priority: 10, aliases: &[],
                category: Category::Info,
                usage: "ohelp | ohelp -list | ohelp <name> | ohelp -desc <text>",
                description: "Object reference: browse and search the object registry.",
                parse: parse_ohelp,
            },
            CommandDef {
                name: "quit", priority: 10, aliases: &["exit"],
                category: Category::Info,
                usage: "quit",
                description: "Quit the game.",
                parse: |_| Ok(Command::Quit),
            },
            CommandDef {
                name: "shutdown", priority: 20, aliases: &[],
                category: Category::Admin,
                usage: "shutdown",
                description: "Shut down the game and gateway. [Admin]",
                parse: |_| Ok(Command::Shutdown),
            },
            CommandDef {
                name: "reboot", priority: 20, aliases: &[],
                category: Category::Admin,
                usage: "reboot [refresh]",
                description: "Restart the game. 'reboot refresh' resets all players to home. [Admin|Dev]",
                parse: parse_reboot,
            },
            CommandDef {
                name: "teleport", priority: 20, aliases: &["goto"],
                category: Category::Admin,
                usage: "teleport <room_id>  |  teleport <q> <r> <area_id>",
                description: "Teleport to a room or outdoor area. [Admin]",
                parse: parse_teleport,
            },
        ])
    }

    // Lookup order:
    //   1. Exact name match
    //   2. Exact alias match
    //   3. Prefix match on name — all matches sorted by priority, lowest wins.
    //      Ties (same priority, multiple matches) are a configuration error → AmbiguousCommand.
    pub fn find(&self, verb: &str) -> Result<&CommandDef, ParseError> {
        if let Some(def) = self.0.iter().find(|d| d.name == verb) {
            return Ok(def);
        }
        if let Some(def) = self.0.iter().find(|d| d.aliases.contains(&verb)) {
            return Ok(def);
        }

        let mut matches: Vec<&CommandDef> = self.0.iter()
            .filter(|d| d.name.starts_with(verb))
            .collect();

        match matches.len() {
            0 => Err(ParseError::UnknownCommand(verb.to_string())),
            1 => Ok(matches[0]),
            _ => {
                matches.sort_by_key(|d| d.priority);
                // If the top two share the same priority, neither wins — configuration error.
                if matches[0].priority == matches[1].priority {
                    Err(ParseError::AmbiguousCommand(verb.to_string()))
                } else {
                    Ok(matches[0])
                }
            }
        }
    }

    pub fn all(&self) -> &[CommandDef] {
        &self.0
    }
}

// --- Shared argument helper ---

// Matches input to a Direction. Resolution order:
// 1. Exact full-name match ("north", "northeast", etc.)
// 2. Canonical single/dual-char alias ("n", "ne", "nw", "s", "se", "sw", "e", "w", "u", "d")
// 3. Unambiguous prefix of a full name — only if exactly one full name starts with input.
pub fn prefix_match_direction(input: &str) -> Option<Direction> {
    const DIRS: &[(&str, Direction)] = &[
        ("north",     Direction::North),
        ("northeast", Direction::NorthEast),
        ("northwest", Direction::NorthWest),
        ("south",     Direction::South),
        ("southeast", Direction::SouthEast),
        ("southwest", Direction::SouthWest),
        ("east",      Direction::East),
        ("west",      Direction::West),
        ("up",        Direction::Up),
        ("down",      Direction::Down),
    ];
    if let Some(&(_, dir)) = DIRS.iter().find(|(n, _)| *n == input) {
        return Some(dir);
    }
    let dir = match input {
        "n"  => Direction::North,
        "ne" => Direction::NorthEast,
        "nw" => Direction::NorthWest,
        "s"  => Direction::South,
        "se" => Direction::SouthEast,
        "sw" => Direction::SouthWest,
        "e"  => Direction::East,
        "w"  => Direction::West,
        "u"  => Direction::Up,
        "d"  => Direction::Down,
        _    => {
            let mut iter = DIRS.iter().filter(|(name, _)| name.starts_with(input));
            let first = iter.next()?;
            return if iter.next().is_none() { Some(first.1) } else { None };
        }
    };
    Some(dir)
}

// --- Individual argument parsers ---

fn parse_look(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Ok(Command::Look(None));
    }
    // Strip optional "at " prefix: `look at forge` → examine "forge"
    let target = rest.strip_prefix("at ").unwrap_or(rest).trim();
    if target.is_empty() {
        return Ok(Command::Look(None));
    }
    match prefix_match_direction(target) {
        Some(dir) => Ok(Command::Look(Some(dir))),
        None      => Ok(Command::Examine(target.to_string())),
    }
}

fn parse_examine(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Examine what?".to_string()));
    }
    let target = rest.strip_prefix("at ").unwrap_or(rest).trim();
    Ok(Command::Examine(target.to_string()))
}

fn parse_get(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Get what?".to_string()));
    }
    Ok(Command::Get(rest.to_string()))
}

fn parse_drop(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Drop what?".to_string()));
    }
    Ok(Command::Drop(rest.to_string()))
}

fn parse_read(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Read what?".to_string()));
    }
    Ok(Command::Read(rest.to_string()))
}

fn parse_eat(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Eat what?".to_string()));
    }
    Ok(Command::Eat(rest.to_string()))
}

fn parse_drink(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Drink what?".to_string()));
    }
    Ok(Command::Drink(rest.to_string()))
}

fn parse_use_item(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Use what?".to_string()));
    }
    Ok(Command::UseItem(rest.to_string()))
}

fn parse_wield(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Wield what?".to_string()));
    }
    Ok(Command::Wield(rest.to_string()))
}

fn parse_wear(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Wear what?".to_string()));
    }
    Ok(Command::Wear(rest.to_string()))
}

fn parse_remove(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingTarget("Remove what?".to_string()));
    }
    Ok(Command::Remove(rest.to_string()))
}

fn parse_go(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingDirection);
    }
    match prefix_match_direction(rest) {
        Some(dir) => Ok(Command::Go(dir)),
        None      => Err(ParseError::UnknownDirection(rest.to_string())),
    }
}

fn parse_enter(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::MissingDirection);
    }
    match prefix_match_direction(rest) {
        Some(dir) => Ok(Command::Enter(dir)),
        None      => Err(ParseError::UnknownDirection(rest.to_string())),
    }
}

fn parse_help(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        Ok(Command::Help(None))
    } else {
        Ok(Command::Help(Some(rest.to_string())))
    }
}

fn parse_reboot(rest: &str) -> Result<Command, ParseError> {
    match rest {
        ""        => Ok(Command::Reboot),
        "refresh" => Ok(Command::RebootRefresh),
        _         => Err(ParseError::UnknownCommand(format!("reboot {rest}"))),
    }
}

fn parse_ohelp(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Ok(Command::OHelp(OHelpQuery::Overview));
    }
    if rest == "-list" {
        return Ok(Command::OHelp(OHelpQuery::List));
    }
    if let Some(text) = rest.strip_prefix("-desc ") {
        let text = text.trim();
        if text.is_empty() {
            return Err(ParseError::MissingTarget("ohelp -desc requires search text.".to_string()));
        }
        return Ok(Command::OHelp(OHelpQuery::Desc(text.to_string())));
    }
    Ok(Command::OHelp(OHelpQuery::Search(rest.to_string())))
}

fn parse_teleport(rest: &str) -> Result<Command, ParseError> {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let usage = "Usage: teleport <room_id>  or  teleport <q> <r> <area_id>";
    match parts.as_slice() {
        [id] => {
            let room_id = id.parse::<u32>()
                .map_err(|_| ParseError::MissingTarget(usage.to_string()))?;
            Ok(Command::Teleport(PlayerLocation::room(room_id)))
        }
        [q, r, area_id] => {
            let q  = q.parse::<i32>()
                .map_err(|_| ParseError::MissingTarget(usage.to_string()))?;
            let r  = r.parse::<i32>()
                .map_err(|_| ParseError::MissingTarget(usage.to_string()))?;
            let id = area_id.parse::<u32>()
                .map_err(|_| ParseError::MissingTarget(usage.to_string()))?;
            Ok(Command::Teleport(PlayerLocation::area(HexCoord::new(q, r), id)))
        }
        _ => Err(ParseError::MissingTarget(usage.to_string())),
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> Registry { Registry::build() }

    // Helper to build a minimal CommandDef for registry tests.
    fn def(name: &'static str, priority: u32) -> CommandDef {
        CommandDef {
            name, priority, aliases: &[],
            category: Category::Info,
            usage: "", description: "",
            parse: |_| Ok(Command::Quit),
        }
    }

    // --- exact matches ---
    #[test] fn find_exact_name()  { assert_eq!(reg().find("look").unwrap().name, "look"); }
    #[test] fn find_exact_alias() { assert_eq!(reg().find("?").unwrap().name,    "help"); }
    #[test] fn find_exit_alias()  { assert_eq!(reg().find("exit").unwrap().name, "quit"); }
    #[test] fn find_move_alias()  { assert_eq!(reg().find("move").unwrap().name, "go"); }
    #[test] fn find_i_alias()     { assert_eq!(reg().find("i").unwrap().name,    "inventory"); }
    #[test] fn find_inv_alias()   { assert_eq!(reg().find("inv").unwrap().name,  "inventory"); }
    #[test] fn find_take_alias()  { assert_eq!(reg().find("take").unwrap().name, "get"); }

    // --- prefix → unique match ---
    #[test] fn find_prefix_look()      { assert_eq!(reg().find("lo").unwrap().name,  "look"); }
    #[test] fn find_prefix_quit()      { assert_eq!(reg().find("qu").unwrap().name,  "quit"); }
    #[test] fn find_prefix_help()      { assert_eq!(reg().find("hel").unwrap().name, "help"); }
    #[test] fn find_prefix_examine()   { assert_eq!(reg().find("ex").unwrap().name,  "examine"); }
    #[test] fn find_prefix_get()       { assert_eq!(reg().find("ge").unwrap().name,  "get"); }
    #[test] fn find_prefix_drop()      { assert_eq!(reg().find("dr").unwrap().name,  "drop"); }
    #[test] fn find_prefix_inventory() { assert_eq!(reg().find("inven").unwrap().name, "inventory"); }

    // --- priority resolution: movement beats same-letter item commands ---
    #[test] fn find_g_resolves_go()    { assert_eq!(reg().find("g").unwrap().name,  "go"); }
    #[test] fn find_d_resolves_down()  { assert_eq!(reg().find("d").unwrap().name,  "down"); }
    #[test] fn find_e_resolves_east()  { assert_eq!(reg().find("e").unwrap().name,  "east"); }

    // --- prefix → single-char (no alias needed) ---
    #[test] fn find_l_resolves_look()  { assert_eq!(reg().find("l").unwrap().name, "look"); }
    #[test] fn find_n_resolves_north() { assert_eq!(reg().find("n").unwrap().name, "north"); }
    #[test] fn find_s_resolves_south() { assert_eq!(reg().find("s").unwrap().name, "south"); }
    #[test] fn find_w_resolves_west()  { assert_eq!(reg().find("w").unwrap().name, "west"); }
    #[test] fn find_h_resolves_help()  { assert_eq!(reg().find("h").unwrap().name, "help"); }
    #[test] fn find_q_resolves_quit()  { assert_eq!(reg().find("q").unwrap().name, "quit"); }

    // --- priority resolution ---
    #[test]
    fn priority_resolves_ambiguous_prefix() {
        let reg = Registry(vec![
            def("look",  1),
            def("learn", 2),
            def("leer",  3),
        ]);
        assert_eq!(reg.find("l").unwrap().name,   "look");   // priority 1 wins
        assert_eq!(reg.find("le").unwrap().name,  "learn");  // priority 2 wins
        assert_eq!(reg.find("lee").unwrap().name, "leer");   // only match
        assert_eq!(reg.find("look").unwrap().name,"look");   // exact name
    }

    // --- same-priority tie → error ---
    #[test]
    fn same_priority_tie_returns_ambiguous_error() {
        let reg = Registry(vec![
            def("look",  1),
            def("learn", 1), // same priority — misconfiguration
        ]);
        assert!(matches!(reg.find("l"), Err(ParseError::AmbiguousCommand(_))));
        // Unambiguous prefix still works even with same priority
        assert_eq!(reg.find("loo").unwrap().name,  "look");
        assert_eq!(reg.find("lea").unwrap().name, "learn");
    }

    // --- unknown ---
    #[test]
    fn find_unknown_returns_error() {
        assert!(matches!(reg().find("fly"), Err(ParseError::UnknownCommand(_))));
    }

    // --- prefix_match_direction ---
    #[test] fn pmdir_full()        { assert_eq!(prefix_match_direction("north"),     Some(Direction::North)); }
    #[test] fn pmdir_single_n()    { assert_eq!(prefix_match_direction("n"),         Some(Direction::North)); }
    #[test] fn pmdir_single_s()    { assert_eq!(prefix_match_direction("s"),         Some(Direction::South)); }
    #[test] fn pmdir_alias_ne()    { assert_eq!(prefix_match_direction("ne"),        Some(Direction::NorthEast)); }
    #[test] fn pmdir_alias_nw()    { assert_eq!(prefix_match_direction("nw"),        Some(Direction::NorthWest)); }
    #[test] fn pmdir_alias_se()    { assert_eq!(prefix_match_direction("se"),        Some(Direction::SouthEast)); }
    #[test] fn pmdir_alias_sw()    { assert_eq!(prefix_match_direction("sw"),        Some(Direction::SouthWest)); }
    #[test] fn pmdir_full_ne()     { assert_eq!(prefix_match_direction("northeast"), Some(Direction::NorthEast)); }
    #[test] fn pmdir_east()        { assert_eq!(prefix_match_direction("e"),         Some(Direction::East)); }
    #[test] fn pmdir_unknown()     { assert_eq!(prefix_match_direction("xyz"),       None); }
    // "no" is ambiguous: north, northeast, northwest all start with "no".
    #[test] fn pmdir_ambiguous_no() { assert_eq!(prefix_match_direction("no"), None); }
}
