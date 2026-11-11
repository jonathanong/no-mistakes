use super::*;

pub(crate) fn check_with_graph_and_session(
    root: &Path,
    config: &NoMistakesConfig,
    graph: &DepGraph,
    facts: &CheckFactMap,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<AnalysisDiagnostics> {
    if !prerequisites::complete_value_flow(facts, session)? {
        return Ok(AnalysisDiagnostics::default());
    }
    let root = crate::codebase::ts_resolver::normalize_path(root);
    let mut diagnostics = AnalysisDiagnostics::default();
    let mut baseline_selectors = AnalysisDiagnostics::default();
    let mut compressed_selectors = AnalysisDiagnostics::default();
    let incomplete = traversal::incomplete_sources(graph, facts);
    prerequisites::record_incomplete(&incomplete, facts, session)?;
    for application in config.rule_applications(RULE_ID) {
        let mut chunk = AnalysisDiagnostics::default();
        let result = (|| -> Result<()> {
            let options = config::Selection::new(&root, application)?;
            let platform_exports = options.platform_consumed_exports(graph);
            let filter = RulePathFilter::new(&root, config, application)?;
            let policy = config.effective_rule_analysis(application)?.unanalyzable;
            let files = facts.graph_file_universe();
            let entries = script_entries::ScriptEntries::collect(&root, files);
            let nextjs_roots = nextjs::project_roots(&root, config);
            let bases = extends::ProductionBases::collect(graph, |path| options.production(path));
            let reachability = runtime_reachability::ProductionReachability::collect(
                graph,
                facts,
                |path| options.production(path),
                &incomplete,
            );
            for (file, line, reason) in &incomplete {
                if runtime_reachability::is_runtime_resolved_unknown(reason)
                    && !reachability.reports(file)
                {
                    continue;
                }
                chunk.unanalyzable(policy, uncertain(&root, application, file, *line, reason));
            }
            let chunks: Vec<_> = files
                .par_iter()
                .filter(|file| {
                    crate::codebase::dependencies::extract::is_indexable(file)
                        && options.production(file)
                        && filter.is_match(file)
                })
                .map(|file| {
                    let mut chunk = AnalysisDiagnostics::default();
                    let Some(ts) = facts.get_ts_facts(file) else {
                        return (chunk, selector_advisories::SelectorAdvisories::default());
                    };
                    let Some(symbols) = ts
                        .symbols
                        .as_ref()
                        .filter(|_| ts.parse_error.is_none() && ts.operational_error.is_none())
                    else {
                        return (chunk, selector_advisories::SelectorAdvisories::default());
                    };
                    let local_uses = options
                        .ignore_exports_used_in_file
                        .then(|| used_in_file::exports(ts));
                    let mut functions = std::collections::BTreeSet::new();
                    let mut selectors = selector_advisories::SelectorAdvisories::default();
                    for export in &symbols.exports {
                        if forwarded(symbols, export) {
                            continue;
                        }
                        let name = if matches!(export.kind, ExportKind::Default) {
                            "default"
                        } else {
                            &export.name
                        };
                        if matches!(export.kind, ExportKind::Function | ExportKind::Default)
                            && !functions.insert(name)
                        {
                            continue;
                        }
                        if platform_exports.contains(
                            &crate::codebase::dependencies::graph::NodeId::symbol(file, name),
                        ) || local_uses.as_ref().is_some_and(|uses| uses.contains(name))
                        {
                            continue;
                        }
                        if entries.exempts(file, nextjs::consumed_export(file, &nextjs_roots, name))
                        {
                            continue;
                        }
                        let mut consumers = traversal::consumers(graph, file, name);
                        let undetermined =
                            consumers.settle(&export.kind, |file| options.test(file));
                        if consumers.files.iter().any(|file| !options.test(file)) {
                            continue;
                        }
                        if bases.extends_class(ts, file, export) {
                            continue;
                        }
                        if !consumers.unknown.is_empty() {
                            selectors.insert(graph, file, name, &export.name, &consumers.unknown);
                        } else if !undetermined.is_empty() {
                            let (importer, line, reason) = selector_advisories::undetermined(
                                &root,
                                graph,
                                &undetermined,
                                &export.name,
                            );
                            chunk.unanalyzable(
                                policy,
                                uncertain(&root, application, &importer, line, &reason),
                            );
                        } else if !consumers.files.is_empty()
                            && consumers.files.iter().all(|file| options.test(file))
                        {
                            chunk
                                .findings
                                .push(finding(&root, application, file, export));
                        }
                    }
                    (chunk, selectors)
                })
                .collect();
            let mut selectors = selector_advisories::SelectorAdvisories::default();
            for (completed, groups) in chunks {
                chunk.append(completed);
                selectors.append(groups);
            }
            let (baseline, compressed) = selectors
                .finish(&root)
                .findings(|file, line, reason| uncertain(&root, application, file, line, reason));
            for finding in baseline {
                baseline_selectors.unanalyzable(policy, finding);
            }
            for finding in compressed {
                compressed_selectors.unanalyzable(policy, finding);
            }
            Ok(())
        })();
        if let Some(session) = session {
            session.capture_check_result(RULE_ID, result)?;
        } else {
            result?;
        }
        diagnostics.append(chunk);
    }
    diagnostics.append(selector_advisories::choose(
        &root,
        session,
        baseline_selectors,
        compressed_selectors,
    ));
    crate::codebase::rules::sort_findings(&mut diagnostics.findings);
    crate::codebase::rules::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}
