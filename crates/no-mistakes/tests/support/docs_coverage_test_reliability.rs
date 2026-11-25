use super::{read, repo_root};

#[test]
fn test_reliability_rules_are_in_the_public_rule_inventory() {
    let root = repo_root();
    for rule in ["test-no-retries", "playwright-no-serial"] {
        for path in [
            "docs/rules/README.md",
            "docs/configuration/rules.md",
            "docs/cli/check.md",
            "docs/node-api.md",
            "docs/feature-parity.md",
        ] {
            assert!(read(&root.join(path)).contains(rule), "{rule}: {path}");
        }
    }
}
