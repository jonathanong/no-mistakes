use super::load;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn sgconfig_upgrades_a_rule_file_before_that_file_is_parsed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/ast-grep-ignore-freshness/fixture/listed-with-config");
    let tracked = [
        "packages/widgets/rules/orders.yml",
        "packages/widgets/sgconfig.yml",
        "packages/widgets/src/items.ts",
        "src/orders.ts",
    ];
    let tracked_owned: Vec<String> = tracked.iter().map(|item| (*item).to_string()).collect();
    let tracked_set: BTreeSet<&str> = tracked.iter().copied().collect();
    let files = vec![
        "packages/widgets/sgconfig.yml".to_string(),
        "packages/widgets/rules".to_string(),
    ];
    let (parsed, findings) = load(&root, &files, &tracked_owned, &tracked_set);
    assert!(findings.is_empty());
    let orders = parsed
        .iter()
        .find(|config| config.relative == "packages/widgets/rules/orders.yml")
        .expect("orders rule");
    assert_eq!(orders.project, "packages/widgets");
}
