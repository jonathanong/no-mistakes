use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::ts_source::facts::TsFileFacts;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;
use std::sync::Arc;

mod plan;
mod playwright_source;
mod program;

use plan::{requires_parse, should_store_source, ts_source};
pub(crate) use program::collect_file_facts_from_program;

pub(crate) fn is_mdx_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("mdx")
}
pub(crate) fn collect_file_facts_with_sources(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Option<CheckFileFacts> {
    let source = match sources.read_path(path) {
        Ok(source) => source,
        Err(err) => {
            let parse_error = format!("failed to read {}: {err}", path.display());
            return Some(CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    parse_error: Some(parse_error.clone()),
                    ..TsFileFacts::default()
                }),
                parse_error: Some(parse_error),
                ..CheckFileFacts::default()
            });
        }
    };
    if plan.storybook && path.extension().and_then(|ext| ext.to_str()) == Some("mdx") {
        let stored_source = should_store_source(plan).then(|| std::sync::Arc::clone(&source));
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(stored_source.clone())),
            source: stored_source,
            storybook: Some(crate::codebase::storybook::extract_mdx_source(&source)),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source && !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts {
            ts: Arc::new(ts_source(Some(std::sync::Arc::clone(&source)))),
            source: Some(std::sync::Arc::clone(&source)),
            ..CheckFileFacts::default()
        });
    }
    if !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts::default());
    }
    let source_type = match SourceType::from_path(path) {
        Ok(source_type) => source_type,
        Err(_) => {
            let stored_source = should_store_source(plan).then(|| std::sync::Arc::clone(&source));
            let parse_error = format!("unsupported file type: {}", path.display());
            return Some(CheckFileFacts {
                ts: Arc::new(TsFileFacts {
                    parse_error: Some(parse_error.clone()),
                    source: stored_source.as_deref().map(str::to_owned),
                    ..TsFileFacts::default()
                }),
                source: stored_source,
                parse_error: Some(parse_error),
                ..CheckFileFacts::default()
            });
        }
    };
    let allocator = Allocator::default();
    #[cfg(any(test, feature = "test-instrumentation"))]
    crate::ast::record_parse_path(path);
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        let parse_error =
            crate::codebase::ts_source::format_parse_diagnostic(path, &parsed.diagnostics);
        let stored_source = should_store_source(plan).then(|| std::sync::Arc::clone(&source));
        let ts = super::file_parse_error::ts_facts(
            plan,
            stored_source.clone(),
            &parsed.program,
            parse_error.clone(),
        );
        let integration_runner_config = plan.integration_runner_configs.as_ref().and_then(|plan| {
            plan.parse_error(
                path,
                format!("failed to parse {}: {parse_error}", path.display()),
            )
        });
        return Some(CheckFileFacts {
            ts: ts.into(),
            source: stored_source,
            integration_runner_config,
            parse_error: Some(parse_error),
            parsed: true,
            ..CheckFileFacts::default()
        });
    }
    let mut facts =
        collect_file_facts_from_program(root, path, plan, playwright, &source, &parsed.program);
    if should_store_source(plan) {
        std::sync::Arc::make_mut(&mut facts.ts).source = Some(source.to_string());
        facts.source = Some(source);
    }
    Some(facts)
}
