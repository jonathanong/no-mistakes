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

fn require_complete_value_flow(files: &[PathBuf], facts: Option<&dyn TsFactLookup>) -> Result<()> {
    for file in files {
        if let Some(reason) = facts
            .and_then(|facts| facts.get_ts_facts(file))
            .and_then(|facts| facts.value_flow.incomplete_reason.as_ref())
        {
            return Err(crate::codebase::check_outcome::CheckInputFailure::source(
                file.clone(),
                format!("{}: {reason}", file.display()),
            )
            .into());
        }
    }
    Ok(())
}
