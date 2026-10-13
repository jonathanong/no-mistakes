#![cfg(test)]
use std::collections::HashSet;
use std::path::Path;
fn collect_items(root: &Path, values: &[String], limit: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        let normalized = value.trim().replace("\\", "/");
        if normalized.is_empty() { continue; }
        let joined = root.join(&normalized);
        let is_file = joined.is_file();
        let is_dir = joined.is_dir();
        if !is_file && !is_dir { continue; }
        if value.starts_with("kind0") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind1") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind2") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind3") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind4") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind5") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind6") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind7") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind8") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind9") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind10") {
            result.push(format!("item-{}", value));
        }
        if value.starts_with("kind11") {
            result.push(format!("item-{}", value));
        }
        let path_text = joined.to_string_lossy().to_string();
        if seen.insert(path_text.clone()) { result.push(path_text); }
        if result.len() >= limit { break; }
    }
    result.sort();
    result.dedup();
    result
}
