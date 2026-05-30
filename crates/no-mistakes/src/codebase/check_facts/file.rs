use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::dependencies::extract::extract_imports_from_program;
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::extract_symbols_from_program;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

pub(crate) fn collect_file_facts(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    playwright: Option<&PlaywrightFactPlan>,
) -> Option<CheckFileFacts> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            return Some(CheckFileFacts {
                parse_error: Some(format!("failed to read {}: {err}", path.display())),
                ..CheckFileFacts::default()
            });
        }
    };
    if plan.storybook && path.extension().and_then(|ext| ext.to_str()) == Some("mdx") {
        let stored_source = should_store_source(plan).then_some(source.clone());
        return Some(CheckFileFacts {
            ts: ts_source(stored_source.clone()),
            source: stored_source,
            storybook: Some(crate::codebase::storybook::extract_mdx_source(&source)),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source && !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts {
            ts: ts_source(Some(source.clone())),
            source: Some(source),
            ..CheckFileFacts::default()
        });
    }
    if !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts::default());
    }
    let source_type = match SourceType::from_path(path) {
        Ok(source_type) => source_type,
        Err(_) => {
            let stored_source = should_store_source(plan).then_some(source);
            return Some(CheckFileFacts {
                ts: ts_source(stored_source.clone()),
                source: stored_source,
                parse_error: Some(format!("unsupported file type: {}", path.display())),
                ..CheckFileFacts::default()
            });
        }
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        let parse_error = parsed
            .errors
            .first()
            .map(|error| format!("{error:?}"))
            .unwrap_or("parser panicked without diagnostic details".to_string());
        let stored_source = should_store_source(plan).then_some(source);
        return Some(CheckFileFacts {
            ts: ts_source(stored_source.clone()),
            source: stored_source,
            parse_error: Some(parse_error),
            ..CheckFileFacts::default()
        });
    }
    let program = &parsed.program;
    let imports = if plan.imports {
        extract_imports_from_program(program)
    } else {
        Vec::new()
    };
    let symbols = if plan.symbols {
        Some(extract_symbols_from_program(program, &source))
    } else {
        None
    };
    let react = if plan.react {
        Some(crate::react_traits::analyze::file::analyze_program(
            path, root, &source, program,
        ))
    } else {
        None
    };
    let queue = if plan.queue {
        Some(crate::queue::extract::extract_program_with_factories(
            path,
            &source,
            program,
            &plan.queue_factory_names,
        ))
    } else {
        None
    };
    let integration = plan
        .integration
        .then(|| crate::integration_tests::analysis::analyze_program(path, program, &source));
    let dynamic_imports = if plan.dynamic_imports {
        Some(
            crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::extract_program(
                &source, program,
            ),
        )
    } else {
        None
    };
    let nextjs_caching = plan.nextjs_caching.then(|| {
        crate::codebase::rules::nextjs_no_caching::extract_program(path, &source, program)
    });
    let storybook = plan
        .storybook
        .then(|| crate::codebase::storybook::extract_program(&source, program));
    let playwright = playwright.and_then(|playwright| {
        let test_id_attributes = playwright.test_id_attributes_by_path.get(path)?;
        Some(super::PlaywrightTestFacts {
            urls:
                crate::playwright::playwright_urls::extract_playwright_url_occurrences_from_program(
                    program,
                    &source,
                    &playwright.navigation_helpers,
                ),
            selectors:
                crate::playwright::selectors::extract_playwright_selector_occurrences_from_program(
                    program,
                    &source,
                    &playwright.selector_regexes,
                    test_id_attributes,
                ),
            text_locators:
                crate::playwright::selectors::extract_playwright_text_locator_occurrences_from_program(
                    program,
                    &source,
                ),
        })
    });
    let ts = TsFileFacts {
        source: should_store_source(plan).then_some(source.clone()),
        imports,
        symbols: symbols.clone(),
        queue_project: queue,
        react_components: react
            .as_ref()
            .map(|analysis| analysis.components.clone())
            .unwrap_or_default(),
        ..Default::default()
    };
    Some(CheckFileFacts {
        ts,
        source: should_store_source(plan).then_some(source),
        symbols,
        react,
        integration,
        dynamic_imports,
        nextjs_caching,
        storybook,
        playwright,
        parse_error: None,
        parsed: true,
    })
}

fn should_store_source(plan: &CheckFactPlan) -> bool {
    plan.source || plan.raw_source
}

fn ts_source(source: Option<String>) -> TsFileFacts {
    TsFileFacts {
        source,
        ..Default::default()
    }
}

fn requires_parse(
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> bool {
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || playwright.is_some_and(|plan| plan.test_id_attributes_by_path.contains_key(path))
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
