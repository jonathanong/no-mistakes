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
        return Some(CheckFileFacts {
            source: should_store_source(plan).then_some(source.clone()),
            storybook: Some(crate::codebase::storybook::extract_mdx_source(&source)),
            ..CheckFileFacts::default()
        });
    }
    if plan.raw_source && !requires_parse(plan, path, playwright) {
        return Some(CheckFileFacts {
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
            return Some(CheckFileFacts {
                source: should_store_source(plan).then_some(source),
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
        return Some(CheckFileFacts {
            source: should_store_source(plan).then_some(source),
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
    let integration = if plan.integration {
        Some(crate::integration_tests::analysis::analyze_program(
            path, program, &source,
        ))
    } else {
        None
    };
    let dynamic_imports = if plan.dynamic_imports {
        Some(
            crate::codebase::rules::test_no_unmocked_dynamic_imports::ast::extract_program(
                &source, program,
            ),
        )
    } else {
        None
    };
    let nextjs_caching = if plan.nextjs_caching {
        Some(crate::codebase::rules::nextjs_no_caching::extract_program(
            path, &source, program,
        ))
    } else {
        None
    };
    let storybook = if plan.storybook {
        Some(crate::codebase::storybook::extract_program(
            &source, program,
        ))
    } else {
        None
    };
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
        imports: imports.clone(),
        symbols: symbols.clone(),
        queue_project: queue.clone(),
        react_components: react
            .as_ref()
            .map(|analysis| analysis.components.clone())
            .unwrap_or_default(),
        ..Default::default()
    };
    Some(CheckFileFacts {
        ts,
        source: should_store_source(plan).then_some(source),
        imports,
        symbols,
        react,
        queue,
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

fn requires_parse(
    plan: &CheckFactPlan,
    path: &Path,
    playwright: Option<&PlaywrightFactPlan>,
) -> bool {
    let playwright_file =
        playwright.is_some_and(|plan| plan.test_id_attributes_by_path.contains_key(path));
    plan.imports
        || plan.symbols
        || plan.react
        || plan.queue
        || plan.integration
        || plan.dynamic_imports
        || plan.nextjs_caching
        || plan.storybook
        || playwright_file
        || plan.source
        || (!plan.raw_source && playwright.is_none())
}
