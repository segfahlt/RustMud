pub struct WorldMap {
    rows:  Vec<String>,
    width: usize,
}

impl WorldMap {
    pub fn empty() -> Self {
        WorldMap {
            rows:  vec!["[world map unavailable — run worldgen/azgaar_parse.py first]".to_string()],
            width: 66,
        }
    }

    pub fn from_rows(rows: Vec<String>) -> Self {
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        WorldMap { rows, width }
    }

    pub fn render(&self) -> String {
        let border = format!("+{}+", "-".repeat(self.width));
        let mut out = String::with_capacity((self.width + 40) * (self.rows.len() + 4));
        out.push_str("{Y}World Geological Survey \u{2014} The Eye (classified){/}\n");
        out.push_str(&border);
        out.push('\n');
        for row in &self.rows {
            out.push('|');
            out.push_str(&colorize_map_row(row));
            // pad to width (color tags don't count as display chars)
            let visible_len = row.chars().count();
            for _ in visible_len..self.width {
                out.push(' ');
            }
            out.push_str("{/}|\n");
        }
        out.push_str(&border);
        out.push('\n');
        out.push_str(LEGEND);
        out
    }
}

/// Returns the color tag for a map character, or "" for neutral/space.
fn map_color(ch: char) -> &'static str {
    match ch {
        '^'        => "{W}",  // Highland Barrens — bright white peaks
        'T'        => "{G}",  // Deep Canopy — bright green
        '%'        => "{g}",  // Canopy Zone — green
        '&'        => "{g}",  // Verdant Zone — green
        '"'        => "{G}",  // Farmland — bright green
        ','        => "{y}",  // Scrubland — yellow
        '.'        => "{K}",  // Scoured Flats — dark gray
        ':'        => "{K}",  // Ash Fields — dark gray
        'w'        => "{c}",  // Brine Flats — cyan
        '~'        => "{b}",  // Ocean — blue
        'o'        => "{b}",  // Lake — blue
        '*'        => "{Y}",  // Firstfall — bright yellow (special)
        _          => "",     // spaces, borders, unknown
    }
}

/// Wraps each run of same-colored characters in a single color tag pair.
fn colorize_map_row(row: &str) -> String {
    let mut out = String::with_capacity(row.len() + 32);
    let mut current_tag: &str = "";

    for ch in row.chars() {
        let tag = map_color(ch);
        if tag != current_tag {
            if !current_tag.is_empty() {
                out.push_str("{/}");
            }
            if !tag.is_empty() {
                out.push_str(tag);
            }
            current_tag = tag;
        }
        out.push(ch);
    }
    if !current_tag.is_empty() {
        out.push_str("{/}");
    }
    out
}

const LEGEND: &str = concat!(
    "  {W}^{/} Highland Barrens   {y},{/} Scrubland     {g}%{/} Canopy Zone   {G}T{/} Deep Canopy\n",
    "  {g}&{/} Verdant Zone       {c}w{/} Brine Flats   {K}.{/} Scoured Flats {K}:{/} Ash Fields\n",
    "  {b}~{/} Ocean              {b}o{/} Lake          {Y}*{/} Firstfall\n",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorize_map_row_ocean() {
        let out = colorize_map_row("~~~");
        assert!(out.contains("{b}"));
        assert!(out.contains("{/}"));
    }

    #[test]
    fn colorize_map_row_mountains() {
        let out = colorize_map_row("^^^");
        assert!(out.contains("{W}"));
    }

    #[test]
    fn colorize_map_row_merges_same_color() {
        // Two adjacent ocean chars should share one color tag, not two
        let out = colorize_map_row("~~");
        assert_eq!(out.matches("{b}").count(), 1);
    }

    #[test]
    fn colorize_map_row_firstfall_yellow() {
        let out = colorize_map_row("*");
        assert!(out.contains("{Y}"));
    }

    #[test]
    fn render_contains_header_and_border() {
        let map = WorldMap::from_rows(vec!["^^^".to_string()]);
        let out = map.render();
        assert!(out.contains("The Eye"));
        assert!(out.contains('+'));
    }
}
