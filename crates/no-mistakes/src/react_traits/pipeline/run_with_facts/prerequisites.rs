use super::*;

type Sources<'a> = HashMap<PathBuf, &'a crate::codebase::check_facts::CheckFileFacts>;
pub(super) fn source_index(shared: &crate::codebase::check_facts::CheckFactMap) -> Sources<'_> {
    shared
        .ts
        .iter()
        .map(|(path, facts)| {
            (
                crate::codebase::ts_source::normalize_discovery_path(path),
                facts.as_ref(),
            )
        })
        .collect()
}

pub(super) fn child_failures<'a>(
    facts: &ComponentFacts,
    root: &Path,
    sources: &Sources<'a>,
) -> Vec<(PathBuf, &'a str)> {
    let mut failures = Vec::new();
    let mut pending = facts.children.iter().collect::<Vec<_>>();
    let mut seen = HashSet::new();
    while let Some(child) = pending.pop() {
        let path = crate::codebase::ts_source::normalize_discovery_path(&root.join(&child.file));
        if !seen.insert((path.clone(), child.name.as_str())) {
            continue;
        }
        let Some(facts) = sources.get(&path) else {
            continue;
        };
        if let Some(error) = &facts.parse_error {
            failures.push((path, error.as_str()));
            continue;
        }
        if let Some(component) = facts.react.as_ref().and_then(|analysis| {
            analysis
                .components
                .iter()
                .find(|facts| facts.name == child.name)
        }) {
            pending.extend(component.children.iter());
        }
    }
    failures.sort();
    failures.dedup();
    failures
}

pub(super) fn accept(
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    failures: &[(PathBuf, &str)],
) -> Result<bool> {
    for (file, message) in failures {
        match session {
            Some(session) if session.partial_checks_enabled() => {
                session.record_check_failure(crate::codebase::check_outcome::CheckFailure {
                    consumer: "assert-no-fetch".into(),
                    file: Some(file.clone()),
                    property: None,
                    message: format!("failed to parse {}: {message}", file.display()),
                })
            }
            _ => anyhow::bail!("failed to parse {}: {message}", file.display()),
        }
    }
    Ok(failures.is_empty())
}

#[cfg(test)]
mod tests;
