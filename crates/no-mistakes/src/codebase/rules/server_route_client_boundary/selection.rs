use super::paths::{relative_path, route_globset_for_rule, ExcludeMatcher};
use super::*;
use rayon::prelude::*;
use std::{collections::HashSet, path::PathBuf};

pub(super) struct CheckContext<'a> {
    pub root: &'a Path,
    pub config: &'a NoMistakesConfig,
    pub sources: &'a crate::codebase::ts_source::SourceStore,
    pub inferred_roots: Option<&'a crate::codebase::config::InferredRoots>,
    pub defer_suppression: bool,
}

pub(super) fn check_items<T>(
    context: CheckContext<'_>,
    items: &[T],
    path_for: impl Fn(&T) -> &Path + Sync,
    facts_for: impl Fn(&T) -> (&FileFacts, Option<&str>) + Sync,
) -> Result<super::super::analysis_policy::AnalysisDiagnostics>
where
    T: Sync,
{
    let CheckContext {
        root,
        config,
        sources,
        inferred_roots,
        defer_suppression,
    } = context;
    let mut diagnostics = super::super::analysis_policy::AnalysisDiagnostics::default();
    let mut inferred_roots = inferred_roots.cloned().unwrap_or_default();
    for rule in config.rule_applications(RULE_ID) {
        let mut findings = Vec::new();
        let opts: Options = rule.try_rule_options()?;
        let exclude_matcher = ExcludeMatcher::new(&opts.excludes);
        let filter = super::super::path_filter::RulePathFilter::new_with_inferred(
            root,
            config,
            rule,
            &mut inferred_roots,
        );
        let filter = filter?;
        let Some(route_globset) = route_globset_for_rule(config, rule) else {
            continue;
        };
        let route_dirs: HashSet<PathBuf> = items
            .par_iter()
            .filter_map(|item| {
                let path = path_for(item);
                (route_globset.is_match(relative_path(root, path))
                    && !exclude_matcher.is_match(root, path)
                    && filter.is_match(path))
                .then(|| {
                    facts_for(item)
                        .0
                        .has_server_route_shape
                        .then(|| path.parent().map(Path::to_path_buf))
                        .flatten()
                })
                .flatten()
            })
            .collect();
        for item in items {
            let path = path_for(item);
            if !exclude_matcher.is_match(root, path)
                && filter.is_match(path)
                && (route_globset.is_match(relative_path(root, path))
                    || path.ancestors().any(|dir| route_dirs.contains(dir)))
            {
                if let Some(error) = facts_for(item).1 {
                    let finding = match sources.read_path(path) {
                        Err(read_error) => super::super::source_access::read_failure(
                            root,
                            path,
                            RULE_ID,
                            &read_error,
                        ),
                        Ok(_) => super::super::source_access::unanalyzable_source(
                            root, path, RULE_ID, error,
                        ),
                    };
                    findings.push(finding);
                }
            }
        }
        findings.extend(
            items
                .par_iter()
                .filter(|item| {
                    let path = path_for(item);
                    !exclude_matcher.is_match(root, path)
                        && filter.is_match(path)
                        && path.ancestors().any(|dir| route_dirs.contains(dir))
                })
                .flat_map(|item| {
                    let path = path_for(item);
                    if !defer_suppression && facts_for(item).0.disabled {
                        return Vec::new();
                    }
                    client_findings_for_file(root, path, facts_for(item).0)
                })
                .collect::<Vec<_>>(),
        );
        diagnostics.append(super::super::file_diagnostics::application(
            root,
            sources,
            config,
            rule,
            findings,
            defer_suppression,
            super::super::file_diagnostics::InputUncertainty::ReadOrParse,
        )?);
    }
    Ok(super::super::file_diagnostics::sorted(diagnostics))
}
