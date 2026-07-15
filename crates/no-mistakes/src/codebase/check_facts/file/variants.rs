use super::{
    collect_file_facts_from_program, collect_file_facts_from_source, is_mdx_file, requires_parse,
    should_store_source,
};
use crate::codebase::check_facts::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::path::Path;
use std::sync::Arc;

mod errors;
use errors::{fill_parse_errors, read_errors};

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
        Err(error) => return read_errors(path, variants.len(), error),
    };
    let mut results = (0..variants.len()).map(|_| None).collect::<Vec<_>>();
    let mut parse_variants = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        if variant.plan.storybook && is_mdx_file(path)
            || !requires_parse(variant.plan, path, variant.playwright)
        {
            results[index] = collect_file_facts_from_source(
                session,
                variant.root,
                path,
                variant.plan,
                variant.playwright,
                Arc::clone(&source),
            );
        } else {
            parse_variants.push((index, variant));
        }
    }
    if parse_variants.is_empty() {
        return results;
    }
    let legacy = parse_variants.iter().any(|(_, variant)| {
        variant
            .plan
            .legacy_symbol_paths
            .contains(&crate::codebase::ts_resolver::normalize_path(path))
    });
    let collect =
        |program: &oxc_ast::ast::Program<'_>, parsed_source: &str, parse_error: Option<String>| {
            parse_variants
                .iter()
                .map(|(index, variant)| {
                    (
                        *index,
                        collect_variant(
                            path,
                            variant,
                            &source,
                            program,
                            parsed_source,
                            parse_error.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>()
        };
    let collected = if legacy {
        session.with_legacy_symbols_program(path, &source, collect)
    } else {
        session.with_recovered_program(path, &source, collect)
    };
    match collected {
        Ok(collected) => {
            for (index, facts) in collected {
                results[index] = Some(facts);
            }
        }
        Err(error) => fill_parse_errors(&mut results, parse_variants, path, &source, legacy, error),
    }
    results
}

fn collect_variant(
    path: &Path,
    variant: &CheckFactVariant<'_>,
    source: &Arc<str>,
    program: &oxc_ast::ast::Program<'_>,
    parsed_source: &str,
    parse_error: Option<String>,
) -> CheckFileFacts {
    let legacy = variant
        .plan
        .legacy_symbol_paths
        .contains(&crate::codebase::ts_resolver::normalize_path(path));
    if let Some(parse_error) = parse_error {
        return recovered_error_facts(
            path,
            variant.plan,
            source,
            program,
            parsed_source,
            parse_error,
            legacy,
        );
    }
    let mut facts = collect_file_facts_from_program(
        variant.root,
        path,
        variant.plan,
        variant.playwright,
        parsed_source,
        program,
    );
    if should_store_source(variant.plan) {
        Arc::make_mut(&mut facts.ts).source = Some(source.to_string());
        facts.source = Some(Arc::clone(source));
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
    legacy: bool,
) -> CheckFileFacts {
    let stored_source = should_store_source(plan).then(|| Arc::clone(source));
    let mut ts = super::super::file_parse_error::ts_facts(
        plan,
        stored_source.clone(),
        program,
        parse_error.clone(),
    );
    let symbols = (legacy && (plan.symbols || plan.graph.symbols)).then(|| {
        Arc::new(crate::codebase::ts_symbols::extract_symbols_from_program(
            program,
            parsed_source,
        ))
    });
    if let Some(symbols) = &symbols {
        ts.symbols = Some(symbols.as_ref().clone());
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
        symbols,
        integration_runner_config,
        parse_error: Some(parse_error),
        parsed: true,
        ..CheckFileFacts::default()
    }
}
