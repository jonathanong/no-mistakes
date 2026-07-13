use std::path::{Path, PathBuf};

pub(crate) fn discover_test_files(
    root: &Path,
    settings: &crate::playwright::config::Settings,
    playwright: &crate::playwright::playwright_config::PlaywrightConfig,
) -> anyhow::Result<Vec<crate::playwright::analysis::context::DiscoveredTestFile>> {
    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    crate::playwright::analysis::discover::discover_test_files_from_visible(
        root, settings, playwright, &snapshot,
    )
}

pub fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["..", "..", "test-cases"]);
    if parts.len() >= 2 {
        path.push(parts[0]);
        path.push(parts[1]);
        path.push("fixture");
        path.extend(&parts[2..]);
    } else {
        path.extend(parts);
    }
    crate::codebase::ts_resolver::normalize_path(&path)
}

pub fn fixture_source(parts: &[&str]) -> String {
    std::fs::read_to_string(fixture_path(parts)).expect("fixture should be readable")
}
