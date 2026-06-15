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
        let mut out = String::with_capacity((self.width + 3) * (self.rows.len() + 4));
        out.push_str("World Geological Survey — The Eye (classified)\n");
        out.push_str(&border);
        out.push('\n');
        for row in &self.rows {
            out.push('|');
            out.push_str(row);
            for _ in row.len()..self.width {
                out.push(' ');
            }
            out.push_str("|\n");
        }
        out.push_str(&border);
        out.push('\n');
        out.push_str(LEGEND);
        out
    }
}

const LEGEND: &str = concat!(
    "  ^ Highland Barrens   , Scrubland     % Canopy Zone   T Deep Canopy\n",
    "  & Verdant Zone       w Brine Flats   . Scoured Flats : Ash Fields\n",
    "  ~ Ocean              o Lake          * Firstfall\n",
);
