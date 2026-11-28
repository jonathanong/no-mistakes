use super::advisory::{
    check_executed_unresolved_star, is_placeholder_relation, resolved_relations,
    ExecutedStarContext,
};
use super::super::fragment_ast::FragmentStar;
use super::super::fragment_parse::FragmentContext;
use super::super::super::{compile_options, Options};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::SchemaCatalog;

fn test_options() -> super::super::super::CompiledOptions {
    compile_options(&Options {
        import_specifier: "@example/db".to_string(),
        schema_catalog_path: "db/schema.json".to_string(),
        max_columns: Some(2),
        relations: vec!["wide".to_string()],
        ..Default::default()
    })
    .unwrap()
}

fn test_catalog() -> SchemaCatalog {
    SchemaCatalog::from_json(
        r#"{
            "formatVersion": 2,
            "coverage": "complete",
            "tables": {
                "wide": { "columns": { "id": { "dataType": "uuid" } } },
                "narrow": { "columns": { "id": { "dataType": "uuid" } } }
            },
            "views": {
                "allowed_view": { "columns": { "id": { "dataType": "uuid" } } }
            }
        }"#,
    )
    .unwrap()
}

#[test]
fn placeholder_and_resolved_relations_cover_all_branches() {
    assert!(is_placeholder_relation("sql_dynamic_outer.column"));
    assert!(is_placeholder_relation("SQL_PLACEHOLDER_12"));
    assert!(!is_placeholder_relation("sql_placeholder_"));
    assert!(!is_placeholder_relation("sql_placeholder_x"));
    assert!(!is_placeholder_relation("wide"));

    let ctx = FragmentContext {
        ctes: vec!["recent".to_string(), "wide".to_string()],
        derived_aliases: vec!["q".to_string()],
        mutation_target: Some(("public.orders".to_string(), Some("o".to_string()))),
        from_relations: vec![
            ("public.items".to_string(), None),
            ("accounts".to_string(), Some("a".to_string())),
        ],
    };

    let direct = FragmentStar {
        returning: false,
        qualifier: None,
        relation: Some("wide".to_string()),
        within_function: None,
        line: 1,
    };
    assert_eq!(resolved_relations(&direct, &ctx), vec![("wide".to_string(), false)]);

    let ret_qualified = FragmentStar {
        returning: true,
        qualifier: Some("items".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&ret_qualified, &ctx),
        vec![("public.items".to_string(), false)]
    );

    let ret_target = FragmentStar {
        returning: true,
        qualifier: Some("o".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&ret_target, &ctx),
        vec![("public.orders".to_string(), true)]
    );

    let ret_bare = FragmentStar {
        returning: true,
        qualifier: None,
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&ret_bare, &ctx),
        vec![("public.orders".to_string(), true)]
    );

    let sel_qualified = FragmentStar {
        returning: false,
        qualifier: Some("a".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&sel_qualified, &ctx),
        vec![("accounts".to_string(), false)]
    );

    let sel_cte = FragmentStar {
        returning: false,
        qualifier: Some("recent".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&sel_cte, &ctx),
        vec![("recent".to_string(), false)]
    );

    let sel_derived = FragmentStar {
        returning: false,
        qualifier: Some("q".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&sel_derived, &ctx),
        vec![("q".to_string(), false)]
    );

    let sel_unknown = FragmentStar {
        returning: false,
        qualifier: Some("missing".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert!(resolved_relations(&sel_unknown, &ctx).is_empty());

    let sel_bare = FragmentStar {
        returning: false,
        qualifier: None,
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&sel_bare, &ctx),
        vec![
            ("public.items".to_string(), false),
            ("accounts".to_string(), false),
        ]
    );
}

#[test]
fn executed_unresolved_star_records_advisories_and_respects_exemptions() {
    let opts = test_options();
    let catalog = test_catalog();
    let facts = CheckFactMap::default();
    let source = Some("line1\nline2\nline3");

    let star = |relation: &str, within_function: Option<&str>, line: usize| SqlStarProjectionFact {
        relation: relation.to_string(),
        qualified: false,
        within_function: within_function.map(str::to_string),
        line,
    };

    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: false,
            from_variant: true,
        },
        &star("sql_placeholder_1", Some("row_to_json"), 1),
    );
    assert!(facts.sql_verdicts().is_empty());

    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: true,
            from_variant: false,
        },
        &star("sql_placeholder_1", None, 2),
    );
    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: false,
            from_variant: true,
        },
        &star("unknown_table", None, 99),
    );
    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: true,
            from_variant: true,
        },
        &star("allowed_view", None, 1),
    );
    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: false,
            from_variant: true,
        },
        &star("narrow", None, 1),
    );

    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 2);
    assert_eq!(verdicts[0].subcheck, "returning-star");
    assert_eq!(verdicts[0].span, (6, 11));
    assert_eq!(verdicts[1].subcheck, "select-star");
    assert!(verdicts[1].residual_reasons[0].contains("unknown_table"));
}
