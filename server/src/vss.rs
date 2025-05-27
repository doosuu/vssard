use json::JsonValue;
use metadata::Metadata;
use spacetimedb::ReducerContext;

use crate::metadata;

fn parse_vss_datapoint(path: &str, data: &JsonValue) -> Metadata {
    Metadata::new(
        String::from(path),
        data["type"].as_str().map_or(String::new(), String::from),
        data["description"].as_str().map_or(String::new(), String::from),
        data["datatype"].as_str().map_or(String::new(), String::from),
        0,
    )
}

pub fn load_vehicle_signals(ctx: &ReducerContext) {
    let vss_file = include_bytes!("../.././vss.json");
    let file_text = String::from_utf8_lossy(vss_file);

    let json_parse_result = json::parse(&file_text);
    if let Err(reason) = json_parse_result {
        panic!("Error in parsing JSON: {}", reason);
    }

    let vehicle_json = json_parse_result.unwrap();
    let mut siblings: Vec<(&str, &JsonValue)> = Vec::new();

    for root_node in vehicle_json.entries() {
        siblings.push(root_node);
    }

    while !siblings.is_empty() {
        let entry: (&str, &JsonValue) = siblings.pop().unwrap();
        let metadata = parse_vss_datapoint(entry.0, entry.1);

        if metadata.datatype.len() > 0 {
            log::info!("Adding '{}' with type '{:?}'", metadata.path, metadata.datatype);
            let _ = metadata::register_datapoint(ctx, metadata);
        }

        let children = entry.1["children"].entries();

        for child in children {
            siblings.push(child);
        }
    }
}
