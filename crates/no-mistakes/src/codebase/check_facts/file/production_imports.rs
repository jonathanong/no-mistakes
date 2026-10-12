//! Historical production-import syntax projection within the union fact pass.

use super::{CheckFactVariant, CheckFileFacts};
use crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source;
use std::path::Path;
use std::sync::Arc;

pub(super) fn collect(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    plan: &crate::codebase::check_facts::CheckFactPlan,
    playwright: Option<&crate::codebase::check_facts::PlaywrightFactPlan>,
    source: Arc<str>,
    retain_parse: bool,
    context: &crate::codebase::ts_source::facts::TsFactContext,
) -> Option<CheckFileFacts> {
    let result = crate::ast::with_request_parse_cache(|| {
        super::collect_file_fact_variants_from_source_with_session_and_contexts(
            session,
            path,
            source,
            &[CheckFactVariant {
                root: &context.root,
                plan,
                playwright,
            }],
            &[context],
        )
        .into_iter()
        .next()
        .flatten()
    });
    if !retain_parse {
        crate::ast::evict_request_parse_cache_path(path);
    }
    result
}

pub(crate) fn project(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: &Arc<str>,
    variants: &[CheckFactVariant<'_>],
    results: &mut [Option<CheckFileFacts>],
) {
    if !variants
        .iter()
        .any(|variant| variant.plan.production_import_paths.contains(path))
        || crate::ast::legacy_symbols_share_standard_parse(path)
    {
        return;
    }
    // ImportExtractor used exactly the legacy-symbol source type, even for
    // JS, explicit module extensions, declaration files, and metadata. Keep
    // this separate from ordinary graph facts; share its physical parse with
    // any legacy-symbol demand already collected above.
    session.record_work("production_dependencies.legacy_import_projections", 1);
    let imports = session
        .with_legacy_symbols_program(path, source, |program, parsed, _| {
            extract_import_facts_from_program_with_source(program, parsed).imports
        })
        .unwrap_or_default();
    let imports = Arc::new(imports);
    for (variant, result) in variants.iter().zip(results) {
        if variant.plan.production_import_paths.contains(path) {
            let result = result.get_or_insert_with(CheckFileFacts::default);
            result.production_imports = Some(Arc::clone(&imports));
            result.parsed = true;
        }
    }
}
