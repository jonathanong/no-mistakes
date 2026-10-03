use super::{read, repo_root};
use std::collections::BTreeSet;

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
        .split_once("| Runtime export | API |\n")
        .and_then(|(_, rest)| rest.split_once("\n\n").map(|(table, _)| table))
        .expect("docs/node-api.md must contain a complete runtime export inventory table");
    let source_exports = exports.iter().copied().collect::<BTreeSet<_>>();
    let documented_rows = runtime_inventory
        .lines()
        .filter_map(|line| {
            line.strip_prefix("| `")?
                .split_once("` |")
                .map(|(name, api)| (name, api.trim().trim_end_matches('|').trim()))
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
        documented_exports, source_exports,
        "runtime export inventory must exactly match packages/no-mistakes/index.js"
    );
    for export in source_exports {
        assert!(
            runtime_inventory.contains(&format!("| `{export}` |")),
            "docs/node-api.md must map runtime export `{export}`"
        );
    }
}
