use crate::world::Direction;
use super::{Command, ParseError};

// --- Category ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Movement,
    Info,
    Communication,
    Admin,
}

impl Category {
    pub fn label(&self) -> &'static str {
        match self {
            Category::Movement      => "Movement",
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
                usage: "look [direction]",
                description: "Look at your surroundings, or peek in a direction.",
                parse: parse_look,
            },
            CommandDef {
                name: "go", priority: 10, aliases: &["move"],
                category: Category::Movement,
                usage: "go <direction>",
                description: "Move in a direction.",
                parse: parse_go,
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
                name: "help", priority: 10, aliases: &["?"],
                category: Category::Info,
                usage: "help [command]",
                description: "Show available commands, or help for a specific command.",
                parse: parse_help,
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

// Prefix-matches input against the six direction names.
// Returns None if the input is unrecognized or ambiguous.
pub fn prefix_match_direction(input: &str) -> Option<Direction> {
    const DIRS: &[(&str, Direction)] = &[
        ("north", Direction::North),
        ("south", Direction::South),
        ("east",  Direction::East),
        ("west",  Direction::West),
        ("up",    Direction::Up),
        ("down",  Direction::Down),
    ];
    let hits: Vec<Direction> = DIRS.iter()
        .filter(|(name, _)| name.starts_with(input))
        .map(|(_, dir)| *dir)
        .collect();
    if hits.len() == 1 { hits.into_iter().next() } else { None }
}

// --- Individual argument parsers ---

fn parse_look(rest: &str) -> Result<Command, ParseError> {
    if rest.is_empty() {
        return Ok(Command::Look(None));
    }
    match prefix_match_direction(rest) {
        Some(dir) => Ok(Command::Look(Some(dir))),
        None      => Err(ParseError::UnknownDirection(rest.to_string())),
    }
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

    // --- prefix → unique match ---
    #[test] fn find_prefix_look() { assert_eq!(reg().find("lo").unwrap().name,  "look"); }
    #[test] fn find_prefix_quit() { assert_eq!(reg().find("qu").unwrap().name,  "quit"); }
    #[test] fn find_prefix_help() { assert_eq!(reg().find("hel").unwrap().name, "help"); }

    // --- prefix → single-char (no alias needed) ---
    #[test] fn find_l_resolves_look()  { assert_eq!(reg().find("l").unwrap().name, "look"); }
    #[test] fn find_n_resolves_north() { assert_eq!(reg().find("n").unwrap().name, "north"); }
    #[test] fn find_s_resolves_south() { assert_eq!(reg().find("s").unwrap().name, "south"); }
    #[test] fn find_e_resolves_east()  { assert_eq!(reg().find("e").unwrap().name, "east"); }
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
    #[test] fn pmdir_full()    { assert_eq!(prefix_match_direction("north"), Some(Direction::North)); }
    #[test] fn pmdir_single()  { assert_eq!(prefix_match_direction("n"),     Some(Direction::North)); }
    #[test] fn pmdir_partial() { assert_eq!(prefix_match_direction("no"),    Some(Direction::North)); }
    #[test] fn pmdir_east()    { assert_eq!(prefix_match_direction("e"),     Some(Direction::East)); }
    #[test] fn pmdir_unknown() { assert_eq!(prefix_match_direction("xyz"),   None); }
}
