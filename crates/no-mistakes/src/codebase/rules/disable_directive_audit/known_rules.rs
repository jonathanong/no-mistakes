use crate::codebase::rules::rule_ids::configurable_rule_ids;
use std::collections::HashSet;
use std::sync::LazyLock;

const DIRECTIVE_ONLY_IDS: [&str; 3] = [
    "queues-check",
    "unused-disable-directive",
    "invalid-disable-directive",
];

static KNOWN_RULES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| configurable_rule_ids().chain(DIRECTIVE_ONLY_IDS).collect());

pub(super) fn known_rule(rule: &str) -> bool {
    KNOWN_RULES.contains(rule)
}
