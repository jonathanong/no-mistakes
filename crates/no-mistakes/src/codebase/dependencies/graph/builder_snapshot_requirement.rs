fn require_playwright_route_snapshot(
    snapshot: Option<&crate::playwright::fsutil::VisiblePathSnapshot>,
) -> Result<&crate::playwright::fsutil::VisiblePathSnapshot> {
    snapshot
        .ok_or_else(|| anyhow::anyhow!("Playwright graph plan requires a visible-path snapshot"))
}

fn missing_prepared_graph_facts(missing: &[PathBuf]) -> anyhow::Error {
    let paths = missing
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    crate::codebase::check_outcome::CheckInputFailure::source(
        missing[0].clone(),
        format!(
            "prepared graph facts are missing {} indexable file(s): {paths}",
            missing.len()
        ),
    )
    .into()
}
