use std::path::PathBuf;

pub fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["..", "..", "test-cases"]);
    if parts.len() >= 2 {
        path.push(parts[0]);
        path.push(parts[1]);
        path.push("fixture");
        path.extend(&parts[2..]);
    } else {
        path.extend(parts);
    }
    path
}

pub fn fixture_source(parts: &[&str]) -> String {
    std::fs::read_to_string(fixture_path(parts)).expect("fixture should be readable")
}
