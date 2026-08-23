use std::path::PathBuf;

/// Materialize root-level query fixtures to give parser instrumentation unique
/// absolute paths for concurrently running tests.
pub(crate) fn materialize_root_fixture(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/queries")
        .join(name);
    crate::test_support::materialize_saved_fixture(&source)
}
