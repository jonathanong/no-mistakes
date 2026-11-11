use super::{CheckFactVariant, VariantRecovery};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;
use std::sync::Arc;

pub(super) fn read_errors(
    path: &Path,
    variants: &[CheckFactVariant<'_>],
    error: impl std::fmt::Display,
) -> Vec<Option<CheckFileFacts>> {
    variants
        .iter()
        .map(|variant| {
            Some(super::super::super::file_parse_error::read_facts(
                variant.root,
                path,
                variant.plan,
                &error,
            ))
        })
        .collect()
}

pub(super) fn fill_parse_errors(
    results: &mut [Option<CheckFileFacts>],
    variants: Vec<(usize, &CheckFactVariant<'_>)>,
    path: &Path,
    source: &Arc<str>,
    legacy: bool,
    error: anyhow::Error,
) {
    for (index, variant) in variants {
        let stored_source =
            super::should_store_source(variant.plan, path).then(|| Arc::clone(source));
        let parse_error = if legacy {
            error.to_string()
        } else {
            format!("unsupported file type: {}", path.display())
        };
        results[index] = Some(CheckFileFacts {
            sql_source_prerequisites:
                crate::codebase::check_facts::SqlSourcePrerequisites::failed_source(
                    variant.root,
                    path,
                    variant.plan,
                ),
            sql_identifier_syntax: crate::codebase::postgres::sql_identifier_syntax::FileFacts {
                low_applications:
                    crate::codebase::check_facts::sql_source_prerequisites::planned_low_applications(
                        variant.root,
                        path,
                        variant.plan,
                    ),
                ..Default::default()
            },
            ts: Arc::new(TsFileFacts {
                fatal_parse_error: legacy,
                parse_error: Some(parse_error.clone()),
                source: (variant.plan.graph.source || stored_source.is_some())
                    .then(|| Arc::clone(source)),
                ..TsFileFacts::default()
            }),
            source: stored_source,
            parse_error: Some(parse_error),
            server_route_client_boundary: variant
                .plan
                .server_route_client_boundary
                .then(Default::default),
            ..CheckFileFacts::default()
        });
    }
}

pub(super) fn recovered_error_facts(
    path: &Path,
    variant: &CheckFactVariant<'_>,
    source: &Arc<str>,
    program: &oxc_ast::ast::Program<'_>,
    parsed_source: &str,
    parse_error: String,
    recovery: VariantRecovery,
) -> CheckFileFacts {
    let plan = variant.plan;
    let stored_source = super::should_store_source(plan, path).then(|| Arc::clone(source));
    let mut ts = super::super::super::file_parse_error::ts_facts(
        plan,
        (plan.graph.source || stored_source.is_some()).then(|| Arc::clone(source)),
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
        sql_source_prerequisites:
            crate::codebase::check_facts::SqlSourcePrerequisites::failed_source(
                variant.root,
                path,
                plan,
            ),
        lint_suppressions: plan.collect_lint_suppressions(path, program, parsed_source),
        ts: Arc::new(ts),
        source: stored_source,
        symbols: symbols.clone(),
        legacy_symbols: symbols,
        integration_runner_config,
        sql_identifier_syntax: super::super::sql_identifier::invalid(
            variant.root,
            path,
            parsed_source,
            program,
            plan,
        ),
        embedded_sql: super::super::embedded_sql::prepared_embedded_sql(
            variant.root,
            path,
            parsed_source,
            program,
            plan,
        ),
        parse_error: Some(parse_error),
        parsed: true,
        server_route_client_boundary: plan.server_route_client_boundary.then(Default::default),
        ..CheckFileFacts::default()
    }
}
