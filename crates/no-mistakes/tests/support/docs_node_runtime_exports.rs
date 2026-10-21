use super::{read, repo_root};
use std::collections::BTreeSet;

fn table_cells(line: &str) -> Option<Vec<&str>> {
    let row = line.trim().strip_prefix('|')?.strip_suffix('|')?;
    Some(row.split('|').map(str::trim).collect())
}

#[test]
fn node_runtime_exports_have_api_docs() {
    let root = repo_root();
    let source = read(&root.join("packages/no-mistakes/index.js"));
    let docs = read(&root.join("docs/node-api.md"));
    let exports = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("module.exports."))
        .filter_map(|assignment| assignment.split_once(' ').map(|(name, _)| name))
        .collect::<Vec<_>>();
    assert!(
        !exports.is_empty(),
        "runtime export inventory must not be empty"
    );
    let runtime_inventory = docs
        .split_once("## Runtime exports\n")
        .and_then(|(_, rest)| rest.trim_start().split_once("\n\n").map(|(table, _)| table))
        .expect("docs/node-api.md must contain a complete runtime export inventory table");
    let mut rows = runtime_inventory.lines();
    let header = rows
        .next()
        .and_then(table_cells)
        .expect("runtime export table header");
    assert_eq!(header, ["Runtime export", "API"]);
    let separator = rows
        .next()
        .and_then(table_cells)
        .expect("runtime export table separator");
    assert_eq!(separator.len(), 2);
    assert!(separator.iter().all(|cell| {
        !cell.is_empty()
            && cell
                .chars()
                .all(|character| character == '-' || character == ':')
    }));
    let source_exports = exports.iter().copied().collect::<BTreeSet<_>>();
    let documented_rows = rows
        .filter_map(table_cells)
        .filter_map(|cells| {
            let [name, api] = cells.as_slice() else {
                return None;
            };
            Some((name.strip_prefix('`')?.strip_suffix('`')?, *api))
        })
        .collect::<Vec<_>>();
    for (export, api) in &documented_rows {
        assert!(
            !api.is_empty(),
            "runtime export `{export}` needs an API mapping"
        );
    }
    let documented_exports = documented_rows
        .iter()
        .map(|(name, _)| *name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        documented_rows.len(),
        source_exports.len(),
        "runtime export inventory must contain one row per runtime export"
    );
    assert_eq!(
        documented_exports, source_exports,
        "runtime export inventory must exactly match packages/no-mistakes/index.js"
    );
}
