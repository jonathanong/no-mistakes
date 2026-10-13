use std::collections::HashSet;
use std::path::Path;
fn resolve_entries(workspace: &Path, entries: &[String], max_count: usize) -> Vec<String> {
    let mut matches = Vec::new();
    let mut observed = HashSet::new();
    for entry in entries {
        let cleaned = entry.trim().replace("\\", "/");
        if cleaned.is_empty() { continue; }
        let resolved_path = workspace.join(&cleaned);
        let is_regular = resolved_path.is_file();
        let is_folder = resolved_path.is_dir();
        if !is_regular && !is_folder { continue; }
        if entry.starts_with("category0") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category1") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category2") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category3") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category4") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category5") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category6") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category7") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category8") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category9") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category10") {
            matches.push(format!("candidate-{}", entry));
        }
        if entry.starts_with("category11") {
            matches.push(format!("candidate-{}", entry));
        }
        let display_name = resolved_path.to_string_lossy().to_string();
        if observed.insert(display_name.clone()) { matches.push(display_name); }
        if matches.len() >= max_count { break; }
    }
    matches.sort();
    matches.dedup();
    matches
}
