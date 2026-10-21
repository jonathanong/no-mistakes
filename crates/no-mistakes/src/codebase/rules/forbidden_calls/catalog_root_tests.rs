use super::{playwright_roots_configured, vitest_roots_configured};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn load_fixture(name: &str) -> NoMistakesConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-calls")
        .join(name)
        .join("fixture");
    crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap()
}

#[test]
fn catalog_roots_follow_configured_root_kind() {
    let coverage = load_fixture("coverage");
    assert!(!vitest_roots_configured(&coverage));
    assert!(!playwright_roots_configured(&coverage));

    let vitest = load_fixture("explicit-vitest-project");
    assert!(vitest_roots_configured(&vitest));
    assert!(!playwright_roots_configured(&vitest));

    let playwright = load_fixture("explicit-playwright-project");
    assert!(!vitest_roots_configured(&playwright));
    assert!(playwright_roots_configured(&playwright));

    let collection = load_fixture("playwright-collection");
    assert!(!vitest_roots_configured(&collection));
    assert!(playwright_roots_configured(&collection));
}
