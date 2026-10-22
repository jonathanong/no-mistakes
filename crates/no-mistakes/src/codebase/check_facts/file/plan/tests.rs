use super::{ts_extract_context, ts_extract_plan, ts_source};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::ts_source::facts::TsFactPlan;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[test]
fn ts_source_reuses_the_same_arc_pointer() {
    let source: Arc<str> = Arc::from("export const value = 1;\n");
    let first = ts_source(Some(Arc::clone(&source)));
    let second = ts_source(Some(Arc::clone(&source)));

    assert!(Arc::ptr_eq(first.source.as_ref().unwrap(), &source));
    assert!(Arc::ptr_eq(second.source.as_ref().unwrap(), &source));
    assert!(Arc::ptr_eq(
        first.source.as_ref().unwrap(),
        second.source.as_ref().unwrap(),
    ));
}

#[test]
fn ts_source_preserves_missing_source() {
    assert!(ts_source(None).source.is_none());
}

#[test]
fn ts_extract_plan_clears_family_flags_outside_path_scope() {
    let inside = PathBuf::from("src/in.ts");
    let outside = PathBuf::from("src/out.ts");
    let scope = crate::codebase::check_facts::family_scope::FamilyPathScope::Only(
        std::sync::Arc::new([inside.clone()].into_iter().collect()),
    );
    let plan = CheckFactPlan {
        react: true,
        queue: true,
        symbols: true,
        react_paths: scope.clone(),
        queue_paths: scope.clone(),
        symbols_paths: scope,
        ..CheckFactPlan::default()
    };
    let ts = ts_extract_plan(Path::new("/repo"), &plan, &outside, None);
    assert!(!ts.react, "{ts:?}");
    assert!(!ts.queue_project, "{ts:?}");
    assert!(!ts.symbols, "{ts:?}");
    let ts = ts_extract_plan(Path::new("/repo"), &plan, &inside, None);
    assert!(ts.react);
    assert!(ts.queue_project);
    assert!(ts.symbols);
}

#[test]
fn ts_extract_plan_mirrors_collected_ts_plan() {
    let path = Path::new("src/widget.ts");
    let plan = CheckFactPlan {
        imports: true,
        symbols: true,
        react: true,
        queue: true,
        source: true,
        graph: TsFactPlan {
            call_sites: true,
            resources: true,
            ..TsFactPlan::default()
        },
        ..CheckFactPlan::default()
    };

    assert_eq!(
        ts_extract_plan(Path::new("/repo"), &plan, path, None),
        plan.collected_ts_plan()
    );
}

#[test]
fn ts_extract_context_uses_check_root_and_queue_factories() {
    let root = Path::new("/repo");
    let plan = CheckFactPlan {
        queue: true,
        queue_factory_names: vec!["createQueue".to_string()],
        ..CheckFactPlan::default()
    };

    let context = ts_extract_context(root, &plan);

    assert_eq!(context.root, root);
    assert_eq!(
        context.queue_project_factory_names,
        ["createQueue".to_string()]
    );
}

#[test]
fn check_file_facts_reuse_the_shared_ts_extract() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/shared-facts/fixture");
    let file = root.join("src/everything.tsx");
    let plan = CheckFactPlan {
        imports: true,
        symbols: true,
        react: true,
        queue: true,
        source: true,
        graph: TsFactPlan {
            call_sites: true,
            ..TsFactPlan::default()
        },
        ..CheckFactPlan::default()
    };
    let check = crate::codebase::check_facts::tests::collect_file_facts(&root, &file, &plan, None)
        .expect("check facts");
    let shared = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        std::slice::from_ref(&file),
        plan.collected_ts_plan(),
        &ts_extract_context(&root, &plan),
    );
    let shared = &shared[&file];

    assert_eq!(check.ts.imports, shared.imports);
    assert_eq!(check.ts.function_calls, shared.function_calls);
    assert_eq!(
        format!("{:?}", check.ts.call_sites),
        format!("{:?}", shared.call_sites)
    );
    assert_eq!(check.ts.symbols, shared.symbols);
    assert_eq!(
        format!("{:?}", check.ts.react_components),
        format!("{:?}", shared.react_components)
    );
    assert_eq!(
        check.ts.queue_project.is_some(),
        shared.queue_project.is_some()
    );
    assert_eq!(check.symbols.as_deref(), check.ts.symbols.as_deref());
    assert!(std::sync::Arc::ptr_eq(
        check.symbols.as_ref().unwrap(),
        check.ts.symbols.as_ref().unwrap(),
    ));
    assert_eq!(
        format!("{:?}", check.react.as_ref().unwrap().components),
        format!("{:?}", check.ts.react_components),
    );
}

#[test]
fn playwright_test_files_request_shared_import_facts() {
    let file = PathBuf::from("src/everything.tsx");
    let plan = CheckFactPlan::default();
    let mut playwright = crate::codebase::check_facts::PlaywrightFactPlan::default();
    playwright.add_file(crate::codebase::check_facts::PlaywrightFactSelection {
        path: file.clone(),
        navigation_helpers: &[],
        selector_wrappers: &[],
        selector_attributes: &["data-testid".to_string()],
        component_selector_attributes: &std::collections::BTreeMap::new(),
        html_ids: false,
        test_id_attributes: &["data-testid".to_string()],
        policy: crate::playwright::playwright_tests::TestPolicy::default(),
        demands_text_imports: true,
    });

    let ts_plan = ts_extract_plan(Path::new("/repo"), &plan, &file, Some(&playwright));

    assert!(ts_plan.imports);
    assert!(ts_plan.function_calls);
    assert!(!plan.collected_ts_plan().imports);
}

#[test]
fn collect_file_facts_from_program_fuses_check_only_walks() {
    let source = include_str!("../program.rs");
    assert!(
        source.contains("collect_fused_check_program"),
        "check facts must walk dynamic imports, nextjs, and storybook together"
    );
    assert!(
        !source.contains("test_no_unmocked_dynamic_imports::ast::extract_program")
            && !source.contains("nextjs_no_caching::extract_program")
            && !source.contains("storybook::extract_program"),
        "check-only extractors must not walk the program again"
    );
}

#[test]
fn unproven_member_spans_follow_rule_path_filters() {
    let source = "export function u(db: { query(text: string): unknown }) {\n  db.query(\"SELECT id FROM users WHERE x = 1\");\n}\n";
    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::from_path(Path::new("query.ts")).unwrap();
    let parsed = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    let root = Path::new("/repo");
    let path = Path::new("/repo/src/legacy/old.ts");
    let profile =
        crate::codebase::postgres::EmbeddedSqlOptions::configured("@example/db", &["pool".into()]);
    let direct = super::super::embedded_sql::prepared_embedded_sql(
        root,
        path,
        source,
        &parsed.program,
        &CheckFactPlan {
            embedded_sql_options: vec![profile.clone()],
            ..Default::default()
        },
    );
    assert_eq!(direct[0].1.unproven_member_query_spans.len(), 1);
    assert!(direct[0].1.record_unproven_spans);
    assert!(direct[0].1.postgres_rule_ids.is_empty());
    let excluded = crate::codebase::postgres::EmbeddedSqlRuleScan::new(
        "postgres-required-predicates",
        &[],
        &["src/legacy/**".into()],
    )
    .unwrap();
    let dropped = super::super::embedded_sql::prepared_embedded_sql(
        root,
        path,
        source,
        &parsed.program,
        &CheckFactPlan {
            embedded_sql_options: vec![profile.clone()],
            embedded_sql_rule_ids: [(profile.clone(), vec![excluded])].into(),
            ..Default::default()
        },
    );
    assert!(dropped[0].1.unproven_member_query_spans.is_empty());
    assert!(!dropped[0].1.record_unproven_spans);
    let included = crate::codebase::postgres::EmbeddedSqlRuleScan::new(
        "postgres-required-predicates",
        &["src/legacy/**".into()],
        &[],
    )
    .unwrap();
    let again = crate::codebase::postgres::EmbeddedSqlRuleScan::new(
        "postgres-required-predicates",
        &[],
        &[],
    )
    .unwrap();
    let counted = super::super::embedded_sql::prepared_embedded_sql(
        root,
        path,
        source,
        &parsed.program,
        &CheckFactPlan {
            embedded_sql_options: vec![profile.clone()],
            embedded_sql_rule_ids: [(profile, vec![included, again])].into(),
            ..Default::default()
        },
    );
    assert_eq!(counted[0].1.unproven_member_query_spans.len(), 1);
    assert!(counted[0].1.record_unproven_spans);
    assert_eq!(
        counted[0].1.postgres_rule_ids,
        ["postgres-required-predicates".to_string()]
    );
}

#[test]
fn playwright_const_urls_share_module_binding_projection_across_variants() {
    let source = crate::playwright::test_support::fixture_source(&[
        "playwright",
        "recall-gaps",
        "const-urls.spec.ts",
    ]);
    let root = Path::new("/repo");
    let file = root.join("const-urls.spec.ts");
    let mut playwright = crate::codebase::check_facts::PlaywrightFactPlan::default();
    for attribute in ["data-testid", "data-test"] {
        playwright.add_file(crate::codebase::check_facts::PlaywrightFactSelection {
            path: file.clone(),
            navigation_helpers: &[],
            selector_wrappers: &[],
            selector_attributes: &[attribute.to_string()],
            component_selector_attributes: &Default::default(),
            html_ids: false,
            test_id_attributes: &[attribute.to_string()],
            policy: Default::default(),
            demands_text_imports: false,
        });
    }
    let plan = CheckFactPlan {
        graph: TsFactPlan {
            module_bindings: true,
            call_arguments: true,
            ..Default::default()
        },
        ..Default::default()
    };
    crate::ast::with_program(&file, &source, |program, source| {
        let baseline = crate::codebase::ts_source::facts::collect_file_facts_from_program(
            &file,
            plan.graph,
            &ts_extract_context(root, &plan),
            source,
            program,
            None,
            None,
        );
        let combined = super::super::program::collect_file_facts_from_program(
            root,
            &file,
            &plan,
            Some(&playwright),
            source,
            program,
            None,
        );
        assert_eq!(
            serde_json::to_value(baseline.module_bindings).unwrap(),
            serde_json::to_value(&combined.ts.module_bindings).unwrap(),
        );
        let variants = combined.playwright.unwrap().all();
        assert_eq!(variants.len(), 2);
        for variant in variants {
            assert!(variant.urls().iter().any(|url| url.value == "/blog/post"));
            assert!(variant
                .urls()
                .iter()
                .any(|url| url.value == "/parameter/${slug}"));
        }
    })
    .unwrap();
}
