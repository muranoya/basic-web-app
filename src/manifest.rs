use serde_json::Value;
use serde_json_path::JsonPath;
use std::fs::File;
use std::io::BufReader;
#[cfg(not(debug_assertions))]
use std::sync::OnceLock;

#[cfg(not(debug_assertions))]
static JS_FILENAME: OnceLock<String> = OnceLock::new();

fn read_manifest(path: &str) -> String {
    let file = File::open("frontend/dist/.vite/manifest.json").unwrap();
    let reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(reader).unwrap();
    let path = JsonPath::parse(path).expect("Invalid JSON Path.");
    let node = path.query(&value).exactly_one();
    node.unwrap().as_str().expect("main is empty.").to_string()
}

#[cfg(debug_assertions)]
pub fn javascript_filename() -> String {
    read_manifest("$[\"src/main.tsx\"].file")
}

#[cfg(not(debug_assertions))]
pub fn javascript_filename() -> String {
    JS_FILENAME
        .get_or_init(|| read_manifest("$[\"src/js/app.js\"].file"))
        .to_string()
}
