use super::{collect_variant, fill_parse_errors, CheckFactVariant};
use crate::codebase::check_facts::CheckFileFacts;
use std::path::Path;
use std::sync::Arc;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) struct VariantParseModes {
    standard: bool,
    legacy_symbols: bool,
}

impl VariantParseModes {
    pub(super) fn for_variant(path: &Path, variant: &CheckFactVariant<'_>) -> Self {
        let normalized = crate::codebase::ts_resolver::normalize_path(path);
        let legacy_symbols = variant.plan.legacy_symbol_paths.contains(&normalized)
            && (variant.plan.symbols || variant.plan.graph.symbols);
        let mut standard_plan = variant.plan.clone();
        standard_plan.symbols = false;
        standard_plan.legacy_symbol_paths.clear();
        // Disable the generic parse fallback so only an actual non-legacy fact
        // demand makes the Standard semantic mode necessary.
        standard_plan.raw_source = true;
        let standard =
            !legacy_symbols || super::requires_parse(&standard_plan, path, variant.playwright);
        Self {
            standard,
            legacy_symbols,
        }
    }
}

pub(super) fn collect_standard_variants(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: &Arc<str>,
    variants: &[(usize, &CheckFactVariant<'_>)],
    modes: &[VariantParseModes],
    results: &mut [Option<CheckFileFacts>],
) {
    let requested = selected(variants, modes, |mode| mode.standard);
    if requested.is_empty() {
        return;
    }
    let collect = |program: &oxc_ast::ast::Program<'_>, parsed: &str, error: Option<String>| {
        requested
            .iter()
            .map(|(index, variant, _)| {
                (
                    *index,
                    collect_variant(path, variant, source, program, parsed, error.clone(), false),
                )
            })
            .collect::<Vec<_>>()
    };
    match session.with_recovered_program(path, source, collect) {
        Ok(collected) => set_results(results, collected),
        Err(error) => fill_parse_errors(
            results,
            requested
                .into_iter()
                .map(|(index, variant, _)| (index, variant))
                .collect(),
            path,
            source,
            false,
            error,
        ),
    }
}

pub(super) fn collect_legacy_variants(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: &Arc<str>,
    variants: Vec<(usize, &CheckFactVariant<'_>)>,
    modes: &[VariantParseModes],
    results: &mut [Option<CheckFileFacts>],
) {
    let requested = selected(&variants, modes, |mode| mode.legacy_symbols);
    if requested.is_empty() {
        return;
    }
    let collect = |program: &oxc_ast::ast::Program<'_>, parsed: &str, error: Option<String>| {
        requested
            .iter()
            .map(|(index, variant, mode)| {
                if mode.standard {
                    let symbols = Arc::new(
                        crate::codebase::ts_symbols::extract_symbols_from_program(program, parsed),
                    );
                    (*index, None, Some(symbols))
                } else {
                    let facts = collect_variant(
                        path,
                        variant,
                        source,
                        program,
                        parsed,
                        error.clone(),
                        true,
                    );
                    (*index, Some(facts), None)
                }
            })
            .collect::<Vec<_>>()
    };
    match session.with_legacy_symbols_program(path, source, collect) {
        Ok(collected) => merge_legacy_results(results, collected),
        Err(error) => set_legacy_errors(results, requested, path, source, error),
    }
}

fn selected<'a>(
    variants: &[(usize, &'a CheckFactVariant<'a>)],
    modes: &[VariantParseModes],
    predicate: impl Fn(VariantParseModes) -> bool,
) -> Vec<(usize, &'a CheckFactVariant<'a>, VariantParseModes)> {
    variants
        .iter()
        .zip(modes)
        .filter(|(_, mode)| predicate(**mode))
        .map(|((index, variant), mode)| (*index, *variant, *mode))
        .collect()
}

fn set_legacy_errors(
    results: &mut [Option<CheckFileFacts>],
    requested: Vec<(usize, &CheckFactVariant<'_>, VariantParseModes)>,
    path: &Path,
    source: &Arc<str>,
    error: anyhow::Error,
) {
    let message = error.to_string();
    let mut legacy_only = Vec::new();
    for (index, variant, mode) in requested {
        if !mode.standard {
            legacy_only.push((index, variant));
            continue;
        }
        let facts = results[index]
            .as_mut()
            .expect("mixed-mode variant must have Standard facts");
        facts.legacy_symbols = None;
        facts.legacy_symbol_parse_error = Some(message.clone());
    }
    fill_parse_errors(
        results,
        legacy_only,
        path,
        source,
        true,
        anyhow::anyhow!(message),
    );
}

fn set_results(results: &mut [Option<CheckFileFacts>], collected: Vec<(usize, CheckFileFacts)>) {
    for (index, facts) in collected {
        results[index] = Some(facts);
    }
}

fn merge_legacy_results(
    results: &mut [Option<CheckFileFacts>],
    collected: Vec<(
        usize,
        Option<CheckFileFacts>,
        Option<Arc<crate::codebase::ts_symbols::FileSymbols>>,
    )>,
) {
    for (index, legacy_only, symbols) in collected {
        if let Some(facts) = legacy_only {
            results[index] = Some(facts);
            continue;
        }
        let facts = results[index]
            .as_mut()
            .expect("mixed-mode variant must have Standard facts");
        facts.legacy_symbols = Some(symbols.expect("mixed-mode variant must have legacy symbols"));
    }
}
