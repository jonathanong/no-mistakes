use super::*;
use no_mistakes::codebase::{rules::analysis_policy::AnalysisDiagnostics, ts_source::SourceStore};
use std::borrow::Cow;

type SelectedSourceCheck =
    fn(&Path, &NoMistakesConfig, &[PathBuf], &SourceStore, bool) -> Result<AnalysisDiagnostics>;

const CHECKS: &[(&str, SelectedSourceCheck)] = &[
    (
        rules::REQUIRED_DOC_SECTION,
        rules::required_local_docs::diagnostics_with_files_and_sources,
    ),
    (
        rules::TEST_NO_DEPENDENCY_PINS,
        rules::test_no_dependency_pins::diagnostics_with_files_and_sources,
    ),
];

pub(super) fn run<'a>(
    root: &Path,
    config: &'a NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<(Cow<'a, NoMistakesConfig>, AnalysisDiagnostics)> {
    let mut remaining = Cow::Borrowed(config);
    let mut diagnostics = AnalysisDiagnostics::default();
    for &(id, check) in CHECKS {
        if config.rule_configured(id) {
            diagnostics.append(no_mistakes::perf_trace::filesystem_rule(id, || {
                check(root, config, files, sources, defer_suppression)
            })?);
            super::handled::mark(remaining.to_mut(), |rule| rule == id);
        }
    }
    Ok((remaining, diagnostics))
}
