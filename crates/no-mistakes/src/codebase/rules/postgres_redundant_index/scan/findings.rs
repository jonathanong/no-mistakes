use super::{CompiledOptions, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use std::collections::BTreeSet;

pub(super) fn stale_allowlist(opts: &CompiledOptions, used: &BTreeSet<String>) -> Vec<RuleFinding> {
    opts.allowed_indexes
        .iter()
        .filter(|entry| !used.contains(*entry))
        .map(|entry| {
            let message = format!("stale postgres-redundant-index allowedIndexes entry: {entry}");
            finding(entry, 1, message, entry)
        })
        .collect()
}

pub(super) fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
