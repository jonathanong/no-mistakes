use super::{read, repo_root, rule_ids_from_source};

#[test]
fn csp_allowed_origins_is_in_the_public_rule_inventory() {
    let rule = "csp-allowed-origins";
    let root = repo_root();
    assert!(rule_ids_from_source(&root).contains(rule));
    for path in [
        "docs/rules/csp-allowed-origins.md",
        "docs/configuration/rules.md",
        "docs/cli/check.md",
        "docs/node-api.md",
        "docs/feature-parity.md",
    ] {
        let content = read(&root.join(path));
        assert!(content.contains(rule), "{path}");
        assert!(content.contains("cspFiles"), "{path}");
    }
}
