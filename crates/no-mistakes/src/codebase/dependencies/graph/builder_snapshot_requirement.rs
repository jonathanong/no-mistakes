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

/// Prepared facts may list files the graph build left out. That is a mismatch
/// unless every omitted file has incomplete value flow: those files cannot
/// contribute call edges, and the files that remain keep the full call plan.
fn supplied_universe_mismatch(primary: &dyn TsFactLookup, requested: &GraphFiles) -> bool {
    let Some(files) = primary.graph_files() else {
        return false;
    };
    if same_graph_universe(files, requested) {
        return false;
    }
    !omitted_files_are_incomplete_value_flow(files, requested, primary)
}

fn omitted_files_are_incomplete_value_flow(
    primary_files: &[PathBuf],
    requested: &GraphFiles,
    facts: &dyn TsFactLookup,
) -> bool {
    let mut seen = 0usize;
    for file in primary_files {
        if requested.contains_visible(file) {
            seen += 1;
            continue;
        }
        let incomplete = facts
            .get_ts_facts(file)
            .is_some_and(|facts| facts.value_flow.incomplete_reason.is_some());
        if !incomplete {
            return false;
        }
    }
    seen == requested.visible_len()
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
