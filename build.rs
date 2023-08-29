use std::env;

fn main() {
    let has_enabled_feature = env::vars().any(|(key, _value)| key.starts_with("CARGO_FEATURE_"));

    if !has_enabled_feature {
        eprintln!("Error: Please enable at least one feature.");
        std::process::exit(1);
    }
}
