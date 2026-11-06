use super::*;
use crate::codebase::ts_resolver::ImportResolverFacade;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub(super) fn accept(
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
    file: &Path,
    message: &str,
) -> Result<bool> {
    match session {
        Some(session) if session.partial_checks_enabled() => {
            session.record_check_failure(crate::codebase::check_outcome::CheckFailure {
                consumer: "integration-test-no-mocks".into(),
                file: Some(file.to_path_buf()),
                property: None,
                message: format!(
                    "failed to parse integration file {}: {message}",
                    file.display()
                ),
            });
            Ok(false)
        }
        _ => anyhow::bail!(
            "failed to parse integration file {}: {message}",
            file.display()
        ),
    }
}

pub(super) fn helper_failures<R: ImportResolverFacade>(
    root: &types::FunctionKey,
    functions: &HashMap<types::FunctionKey, types::FunctionInfo>,
    resolver: &resolve::ImportResolution<'_, R>,
    errors: &HashMap<PathBuf, &str>,
) -> Vec<PathBuf> {
    let mut pending = vec![root.clone()];
    let mut seen = HashSet::new();
    let mut failures = Vec::new();
    while let Some(key) = pending.pop() {
        if !seen.insert(key.clone()) {
            continue;
        }
        let Some(info) = functions.get(&key) else {
            continue;
        };
        let Some(analysis) = resolver.analyses.get(&key.file) else {
            continue;
        };
        for call in &info.calls {
            let binding = match call {
                types::CallTarget::Local(local) | types::CallTarget::Imported { local } => {
                    analysis.imports.get(local)
                }
                types::CallTarget::Namespace { namespace, .. } => analysis.imports.get(namespace),
            };
            if let Some(path) = binding
                .and_then(|binding| resolver.resolver.resolve(&binding.source, &key.file))
                .and_then(|path| resolver.remapper.remap(&path))
            {
                if errors.contains_key(&path) {
                    failures.push(path);
                    continue;
                }
            }
            if let Some(next) = resolve::resolve_call(&key, call, resolver) {
                pending.push(next);
            }
        }
    }
    failures.sort();
    failures.dedup();
    failures
}

#[cfg(test)]
mod tests;
