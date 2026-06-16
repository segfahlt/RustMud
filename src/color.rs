/// Expands `{tag}` color codes in `text` to ANSI escape sequences.
/// When `color` is false, known tags are stripped and unknown tags pass through.
///
/// Tags:
///   Normal:  {k} {r} {g} {y} {b} {m} {c} {w}   (black red green yellow blue magenta cyan white)
///   Bright:  {K} {R} {G} {Y} {B} {M} {C} {W}
///   Style:   {*} bold   {/} reset all
///
/// Unknown tags (e.g. {foo}) are emitted literally so prose braces don't get eaten.
///
/// TODO: derive `color` per-client from TTYPE/MTTS telnet negotiation.
pub fn colorize(text: &str, color: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        rest = &rest[open + 1..];
        match rest.find('}') {
            Some(close) => {
                let tag = &rest[..close];
                rest = &rest[close + 1..];
                match expand_tag(tag, color) {
                    Some(code) => out.push_str(code),
                    None => { out.push('{'); out.push_str(tag); out.push('}'); }
                }
            }
            None => {
                // No closing brace — emit the '{' and leave the rest unchanged.
                out.push('{');
            }
        }
    }
    out.push_str(rest);
    out
}

fn expand_tag(tag: &str, color: bool) -> Option<&'static str> {
    if !color {
        return match tag {
            "k"|"r"|"g"|"y"|"b"|"m"|"c"|"w"|
            "K"|"R"|"G"|"Y"|"B"|"M"|"C"|"W"|
            "*"|"/" => Some(""),
            _ => None,
        };
    }
    Some(match tag {
        // Normal (dim) foreground colors
        "k" => "\x1b[30m",
        "r" => "\x1b[31m",
        "g" => "\x1b[32m",
        "y" => "\x1b[33m",
        "b" => "\x1b[34m",
        "m" => "\x1b[35m",
        "c" => "\x1b[36m",
        "w" => "\x1b[37m",
        // Bright foreground colors
        "K" => "\x1b[90m",
        "R" => "\x1b[91m",
        "G" => "\x1b[92m",
        "Y" => "\x1b[93m",
        "B" => "\x1b[94m",
        "M" => "\x1b[95m",
        "C" => "\x1b[96m",
        "W" => "\x1b[97m",
        // Style
        "*" => "\x1b[1m",
        "/" => "\x1b[0m",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn no_tags_unchanged()     { assert_eq!(colorize("hello", true),  "hello"); }
    #[test] fn no_tags_no_color()      { assert_eq!(colorize("hello", false), "hello"); }

    #[test]
    fn known_tag_expands() {
        assert_eq!(colorize("{r}red{/}", true), "\x1b[31mred\x1b[0m");
    }

    #[test]
    fn known_tag_stripped_when_no_color() {
        assert_eq!(colorize("{r}red{/}", false), "red");
    }

    #[test]
    fn unknown_tag_passes_through() {
        assert_eq!(colorize("{foo} bar", true),  "{foo} bar");
        assert_eq!(colorize("{foo} bar", false), "{foo} bar");
    }

    #[test]
    fn unclosed_brace_passes_through() {
        assert_eq!(colorize("cost {gold", true), "cost {gold");
    }

    #[test]
    fn bright_colors_expand() {
        assert_eq!(colorize("{R}", true), "\x1b[91m");
        assert_eq!(colorize("{W}", true), "\x1b[97m");
    }

    #[test]
    fn bold_and_reset() {
        assert_eq!(colorize("{*}bold{/}", true), "\x1b[1mbold\x1b[0m");
    }

    #[test]
    fn multiple_tags() {
        let result = colorize("{g}Go {r}stop{/}", true);
        assert_eq!(result, "\x1b[32mGo \x1b[31mstop\x1b[0m");
    }

    #[test]
    fn tag_adjacent_to_text() {
        assert_eq!(colorize("a{r}b{/}c", true), "a\x1b[31mb\x1b[0mc");
    }
}
