use schemars::schema_for;
use rustmud::world::loader::ZoneFile;

fn main() {
    let schema = schema_for!(ZoneFile);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
