use super::sql_source_prerequisites::with_failed_source;
use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;
use std::sync::Arc;
mod embedded_sql;
mod plan;
mod playwright_source;
mod prepared_program;
pub(crate) mod production_imports;
mod program;
mod program_walk;
mod sql_identifier;
mod variants;
pub(super) use plan::should_store_source;
use plan::{requires_parse, ts_source};
pub(crate) use program::collect_file_facts_from_program_with_discovery;
pub(crate) use variants::{
    collect_file_fact_variants_from_source_with_session, collect_file_fact_variants_with_session,
    CheckFactVariant,
};
pub(crate) fn is_mdx_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("mdx")
}
pub(crate) fn collect_file_facts_with_request(
    session: super::FactRequest<'_>,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    crate::codebase::fact_panic::isolate(session.session, path, || {
        collect_file_facts_unchecked(session, root, path, plan, playwright, sources, retain_parse)
    })
    .unwrap_or_else(|error| {
        let source = (plan.graph.source || should_store_source(plan, path))
            .then(|| {
                crate::codebase::ts_source::SourceStore::read_prepared_or_open(Some(sources), path)
                    .ok()
            })
            .flatten();
        Some(super::file_parse_error::panic_facts(
            root, path, plan, error, source,
        ))
    })
}

fn collect_file_facts_unchecked(
    session: super::FactRequest<'_>,
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<CheckFileFacts> {
    if plan.raw_source
        && !should_store_source(plan, path)
        && !requires_parse(root, plan, path, playwright)
    {
        return Some(CheckFileFacts::default());
    }
    let read = if crate::codebase::dependencies::extract::is_indexable(path) {
        sources.read_source_path(path)
    } else {
        sources.read_path(path)
    };
    let source = match read {
        Ok(source) => source,
        Err(err) => return Some(super::file_parse_error::read_facts(root, path, plan, err)),
    };
    if plan.production_import_paths.contains(path) {
        return production_imports::collect(
            session,
            root,
            path,
            plan,
            playwright,
            source,
            retain_parse,
        );
    }
    collect_file_facts_from_source(session, root, path, plan, playwright, source, retain_parse)
}
include!("file/source.rs");
