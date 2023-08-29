use std::env;

fn main() {
    let enabled_features: Vec<String> = env::vars()
        .filter_map(|(key, _value)| {
            if key.starts_with("CARGO_FEATURE_") {
                Some(key[14..].to_string())
            } else {
                None
            }
        })
        .collect();

    if enabled_features.is_empty() {
        eprintln!("Error: Please enable at least one feature.");
        std::process::exit(1);
    }
}
