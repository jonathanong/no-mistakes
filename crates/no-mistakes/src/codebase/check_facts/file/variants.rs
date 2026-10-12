use super::{collect_file_facts_from_source, is_mdx_file, requires_parse, should_store_source};
use crate::codebase::check_facts::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::path::Path;
use std::sync::Arc;

mod errors;
use errors::{fill_parse_errors, read_errors};
mod contexts;
pub(crate) use contexts::collect_file_fact_variants_from_source_with_session;
mod parse_modes;
use parse_modes::{collect_legacy_variants, collect_standard_variants, VariantParseModes};

#[cfg(test)]
mod tests;

pub(crate) struct CheckFactVariant<'a> {
    pub(crate) root: &'a Path,
    pub(crate) plan: &'a CheckFactPlan,
    pub(crate) playwright: Option<&'a PlaywrightFactPlan>,
}

pub(crate) fn collect_file_fact_variants_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    variants: &[CheckFactVariant<'_>],
) -> Vec<Option<CheckFileFacts>> {
    let source = match session.read_source(path) {
        Ok(source) => source,
        Err(error) => return read_errors(path, variants, error),
    };
    collect_file_fact_variants_from_source_with_session(session, path, source, variants)
}

pub(crate) fn collect_file_fact_variants_from_source_with_session_and_contexts(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: Arc<str>,
    variants: &[CheckFactVariant<'_>],
    contexts: &[&crate::codebase::ts_source::facts::TsFactContext],
) -> Vec<Option<CheckFileFacts>> {
    assert_eq!(variants.len(), contexts.len());
    let mut results = (0..variants.len()).map(|_| None).collect::<Vec<_>>();
    let mut parse_variants = Vec::new();
    for (index, (variant, context)) in variants.iter().zip(contexts).enumerate() {
        if variant.plan.production_import_paths.contains(path)
            && !crate::codebase::dependencies::extract::is_indexable(path)
            && !(variant.plan.storybook && is_mdx_file(path))
        {
            // Unknown extensions were historically parsed as TypeScript by
            // this rule. Avoid a competing unsupported Standard parse.
            let stored_source = should_store_source(variant.plan).then(|| Arc::clone(&source));
            results[index] = Some(CheckFileFacts {
                ts: Arc::new(super::ts_source(stored_source.clone())),
                source: stored_source,
                ..CheckFileFacts::default()
            });
        } else if variant.plan.storybook && is_mdx_file(path)
            || !requires_parse(variant.plan, path, variant.playwright)
        {
            results[index] = collect_file_facts_from_source(
                session,
                path,
                variant.plan,
                variant.playwright,
                Arc::clone(&source),
                false,
                context,
            );
        } else {
            parse_variants.push((index, variant));
        }
        if variant.plan.production_import_paths.contains(path)
            && !crate::codebase::dependencies::extract::is_indexable(path)
            && variant.plan.legacy_symbol_paths.contains(path)
            && (variant.plan.symbols || variant.plan.graph.symbols)
        {
            parse_variants.push((index, variant));
        }
    }
    if parse_variants.is_empty() {
        super::production_imports::project(session, path, &source, variants, &mut results);
        return results;
    }
    let modes = parse_variants
        .iter()
        .map(|(_, variant)| VariantParseModes::for_variant(path, variant))
        .collect::<Vec<_>>();
    collect_standard_variants(
        session,
        path,
        &source,
        &parse_variants,
        &modes,
        contexts,
        &mut results,
    );
    collect_legacy_variants(
        session,
        path,
        &source,
        parse_variants,
        &modes,
        contexts,
        &mut results,
    );
    super::production_imports::project(session, path, &source, variants, &mut results);
    results
}

struct VariantRecovery {
    symbols: bool,
    fatal_parse_error: bool,
}

struct VariantParseStatus {
    recovery: VariantRecovery,
    parse_error: Option<String>,
}

impl VariantParseStatus {
    fn new(symbols: bool, fatal_parse_error: bool, parse_error: Option<String>) -> Self {
        Self {
            recovery: VariantRecovery {
                symbols,
                fatal_parse_error,
            },
            parse_error,
        }
    }
}

fn collect_variant(
    path: &Path,
    variant: &CheckFactVariant<'_>,
    source: &Arc<str>,
    program: &oxc_ast::ast::Program<'_>,
    parsed_source: &str,
    status: VariantParseStatus,
    context: &crate::codebase::ts_source::facts::TsFactContext,
) -> CheckFileFacts {
    if let Some(parse_error) = status.parse_error {
        return recovered_error_facts(
            path,
            variant.plan,
            source,
            program,
            parsed_source,
            parse_error,
            status.recovery,
        );
    }
    // Fatal parser results always carry a diagnostic and return above.
    let mut facts = super::program::collect_file_facts_from_program_with_context(
        path,
        variant.plan,
        variant.playwright,
        parsed_source,
        program,
        should_store_source(variant.plan).then(|| Arc::clone(source)),
        context,
    );
    if status.recovery.symbols {
        facts.legacy_symbols = facts.symbols.clone();
    }
    facts
}

fn recovered_error_facts(
    path: &Path,
    plan: &CheckFactPlan,
    source: &Arc<str>,
    program: &oxc_ast::ast::Program<'_>,
    parsed_source: &str,
    parse_error: String,
    recovery: VariantRecovery,
) -> CheckFileFacts {
    let stored_source = should_store_source(plan).then(|| Arc::clone(source));
    let mut ts = super::super::file_parse_error::ts_facts(
        plan,
        stored_source.clone(),
        parsed_source,
        program,
        parse_error.clone(),
    );
    ts.fatal_parse_error = recovery.fatal_parse_error;
    let symbols = (recovery.symbols && (plan.symbols || plan.graph.symbols)).then(|| {
        Arc::new(crate::codebase::ts_symbols::extract_symbols_from_program(
            program,
            parsed_source,
        ))
    });
    if let Some(symbols) = &symbols {
        ts.symbols = Some(Arc::clone(symbols));
    }
    let integration_runner_config = plan.integration_runner_configs.as_ref().and_then(|runner| {
        runner.parse_error(
            path,
            format!("failed to parse {}: {parse_error}", path.display()),
        )
    });
    CheckFileFacts {
        ts: Arc::new(ts),
        source: stored_source,
        symbols: symbols.clone(),
        legacy_symbols: symbols,
        integration_runner_config,
        embedded_sql: super::program::prepared_embedded_sql(path, parsed_source, program, plan),
        parse_error: Some(parse_error),
        parsed: true,
        server_route_client_boundary: plan.server_route_client_boundary.then(Default::default),
        ..CheckFileFacts::default()
    }
}
