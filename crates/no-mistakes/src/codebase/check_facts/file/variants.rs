use super::{
    collect_file_facts_from_program, collect_file_facts_from_source, is_mdx_file, requires_parse,
    should_store_source,
};
use crate::codebase::check_facts::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use std::path::Path;
use std::sync::Arc;

mod errors;
use errors::{fill_parse_errors, read_errors, recovered_error_facts};
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

pub(crate) fn collect_file_fact_variants_from_source_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    path: &Path,
    source: Arc<str>,
    variants: &[CheckFactVariant<'_>],
) -> Vec<Option<CheckFileFacts>> {
    let mut results = (0..variants.len()).map(|_| None).collect::<Vec<_>>();
    let mut parse_variants = Vec::new();
    for (index, variant) in variants.iter().enumerate() {
        if variant.plan.production_import_paths.contains(path)
            && !crate::codebase::dependencies::extract::is_indexable(path)
            && !(variant.plan.storybook && is_mdx_file(path))
        {
            // Unknown extensions were historically parsed as TypeScript by
            // this rule. Avoid a competing unsupported Standard parse.
            let stored_source =
                should_store_source(variant.plan, path).then(|| Arc::clone(&source));
            results[index] = Some(CheckFileFacts {
                ts: Arc::new(super::ts_source(stored_source.clone())),
                source: stored_source,
                ..CheckFileFacts::default()
            });
        } else if variant.plan.storybook && is_mdx_file(path)
            || !requires_parse(variant.root, variant.plan, path, variant.playwright)
        {
            results[index] = collect_file_facts_from_source(
                session,
                variant.root,
                path,
                variant.plan,
                variant.playwright,
                Arc::clone(&source),
                false,
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
        &mut results,
    );
    collect_legacy_variants(session, path, &source, parse_variants, &modes, &mut results);
    super::production_imports::project(session, path, &source, variants, &mut results);
    results
}

#[derive(Clone, Copy)]
struct VariantRecovery {
    symbols: bool,
    fatal_parse_error: bool,
}

fn collect_variant(
    path: &Path,
    variant: &CheckFactVariant<'_>,
    source: &Arc<str>,
    program: &oxc_ast::ast::Program<'_>,
    parsed_source: &str,
    parse_error: Option<String>,
    recovery: VariantRecovery,
) -> CheckFileFacts {
    if let Some(parse_error) = parse_error {
        return recovered_error_facts(
            path,
            variant,
            source,
            program,
            parsed_source,
            parse_error,
            recovery,
        );
    }
    // Fatal parser results always carry a diagnostic and return above.
    let mut facts = collect_file_facts_from_program(
        variant.root,
        path,
        variant.plan,
        variant.playwright,
        parsed_source,
        program,
        should_store_source(variant.plan, path).then(|| Arc::clone(source)),
    );
    if recovery.symbols {
        facts.legacy_symbols = facts.symbols.clone();
    }
    facts
}
