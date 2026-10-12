use super::*;

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_facts(root, config, all_files, sources, None)
}

pub(crate) fn check_with_files_sources_and_facts(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
) -> Result<Vec<RuleFinding>> {
    let session = facts.is_none().then(|| {
        crate::codebase::analysis_session::AnalysisSession::new(sources.observer().cloned())
    });
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.try_rule_options()?;
            let target_roots = super::super::target_roots(root, config, rule);
            let skip = super::super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| {
                    super::super::file_allowed_by_roots_and_skip(root, &skip, path, &target_roots)
                })
                .cloned()
                .collect();
            let files = super::super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan_with_sources_and_facts(root, &opts, &files, sources, facts, session.as_deref())
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::super::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn scan_with_sources_and_facts(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let compiled = compile_options(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file_with_sources(root, path, &compiled, sources, facts, session))
        .collect();
    super::super::sort_findings(&mut findings);
    Ok(findings)
}

pub(super) fn check_file_with_sources(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    facts: Option<&dyn crate::codebase::dependencies::graph::TsFactLookup>,
    session: Option<&crate::codebase::analysis_session::AnalysisSession>,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    if !opts.includes(&rel) {
        return Vec::new();
    }
    let Some(content) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    let fallback;
    let ranges = if !opts.patterns.iter().any(|pattern| pattern.multiline) || !is_jsx_file(path) {
        &[][..]
    } else if let Some(facts) = facts {
        // Failure entries are authoritative too; never reopen a prepared parse.
        facts
            .get_ts_facts(path)
            .and_then(|prepared| prepared.jsx_text_ranges.as_deref())
            .unwrap_or_default()
    } else {
        fallback = session
            .map(|session| scan::jsx_text_ranges::collect(path, &content, session))
            .unwrap_or_default();
        &fallback
    };
    scan::check_source_with_ranges(&rel, &content, opts, ranges)
}
