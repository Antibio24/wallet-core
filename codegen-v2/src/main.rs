use libparser::parse;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let path = Path::new("../include/");
    let dir = parse(&path).expect("Failed to parse path");
    let data = serde_json::to_string_pretty(&dir.map).expect("Failed to generate JSON");

    let mut file = File::create("include_manifest.json").expect("Unable to create file");
    file.write_all(data.as_bytes())
        .expect("Unable to write data to file");
}