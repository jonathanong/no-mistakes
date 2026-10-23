use super::super::{TsFactContext, TsFactPlan, TsFileFacts};
use super::program::collect_file_facts_from_program;
use crate::codebase::dependencies::extract::is_indexable;
use std::path::Path;
use std::sync::Arc;

pub(super) fn facts_from_collection_result(result: anyhow::Result<TsFileFacts>) -> TsFileFacts {
    match result {
        Ok(facts) => facts,
        // Indexable extensions are expected to have an OXC source type.
        // Reaching this branch means that allowlist and OXC support drifted.
        Err(error) => failure_facts(error.to_string()),
    }
}

pub(super) fn collect_file_facts_with_sources_and_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    plan: TsFactPlan,
    context: &TsFactContext,
    sources: &crate::codebase::ts_source::SourceStore,
    retain_parse: bool,
) -> Option<TsFileFacts> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(error) => {
            return Some(failure_facts(format!(
                "failed to read {}: {error}",
                path.display()
            )));
        }
    };
    // Project-wide collection filters to known TS/JS extensions. Parse those
    // in standard recovered mode so runner-config analysis can share the same
    // request cache. The direct-file test/support path retains the historical
    // TypeScript fallback for unknown extensions.
    let stored_source = plan.source.then(|| Arc::clone(&source));
    let result = if is_indexable(path) {
        session.with_recovered_program_status(
            path,
            &source,
            |program, parsed, parse_error, panicked| {
                let mut facts = collect_file_facts_from_program(
                    path,
                    plan,
                    context,
                    parsed,
                    program,
                    parse_error,
                    stored_source.clone(),
                );
                facts.fatal_parse_error = panicked;
                facts
            },
        )
    } else {
        session.with_recovered_typescript_program(path, &source, |program, parsed, parse_error| {
            collect_file_facts_from_program(
                path,
                plan,
                context,
                parsed,
                program,
                parse_error,
                stored_source,
            )
        })
    };
    let facts = Some(facts_from_collection_result(result));
    if !retain_parse {
        crate::ast::evict_request_parse_cache_path(path);
    }
    facts
}

fn failure_facts(error: String) -> TsFileFacts {
    // Preserve request-level failures and the legacy parser diagnostic channel.
    TsFileFacts {
        operational_error: Some(error.clone()),
        parse_error: Some(error),
        ..TsFileFacts::default()
    }
}

#[cfg(test)]
mod tests;
