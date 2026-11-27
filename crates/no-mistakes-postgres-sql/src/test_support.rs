use std::path::PathBuf;

pub(crate) fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../no-mistakes/fixtures")
        .join(name)
}

pub(crate) fn rule_fixture_root(rule: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("fixture")
}
