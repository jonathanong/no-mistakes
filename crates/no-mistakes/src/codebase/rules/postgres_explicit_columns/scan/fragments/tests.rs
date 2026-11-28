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
        ..Default::default()
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

    let ret_cte = FragmentStar {
        returning: true,
        qualifier: Some("recent".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&ret_cte, &ctx),
        vec![("recent".to_string(), false)]
    );

    let ret_missing = FragmentStar {
        returning: true,
        qualifier: Some("missing".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert!(resolved_relations(&ret_missing, &ctx).is_empty());

    let ret_bare_none = FragmentStar {
        returning: true,
        qualifier: None,
        relation: None,
        within_function: None,
        line: 1,
    };
    assert!(resolved_relations(&ret_bare_none, &FragmentContext::default()).is_empty());

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

    let dotted_ctx = super::super::fragment_parse::parse_context(
        r#"UPDATE public."audit.log" SET id = 1"#,
    );
    let dotted_star = FragmentStar {
        returning: true,
        qualifier: Some("audit.log".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&dotted_star, &dotted_ctx),
        vec![(r#"public."audit.log""#.to_string(), true)]
    );

    let join_ctx = super::super::fragment_parse::parse_context(
        "UPDATE target SET id = 1 FROM (wide w JOIN narrow n ON w.id = n.id) AS j",
    );
    let join_ret = FragmentStar {
        returning: true,
        qualifier: Some("j".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&join_ret, &join_ctx),
        vec![("wide".to_string(), false), ("narrow".to_string(), false)]
    );
    let join_sel = FragmentStar {
        returning: false,
        qualifier: Some("j".to_string()),
        relation: None,
        within_function: None,
        line: 1,
    };
    assert_eq!(
        resolved_relations(&join_sel, &join_ctx),
        vec![("wide".to_string(), false), ("narrow".to_string(), false)]
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
    check_executed_unresolved_star(
        &facts,
        &opts,
        &catalog,
        ExecutedStarContext {
            rel: "src/a.ts",
            source,
            returning: false,
            from_variant: false,
        },
        &star("sql_placeholder_2", None, 1),
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
        &star("wide", None, 1),
    );

    let verdicts = facts.sql_verdicts();
    assert_eq!(verdicts.len(), 3);
    assert_eq!(verdicts[0].subcheck, "select-star");
    assert_eq!(verdicts[0].span, (0, 5));
    assert_eq!(verdicts[1].subcheck, "returning-star");
    assert_eq!(verdicts[1].span, (6, 11));
    assert_eq!(verdicts[2].subcheck, "select-star");
    assert!(verdicts[2].residual_reasons[0].contains("unknown_table"));
}

#[test]
fn scan_fragments_respects_returning_option_cte_vs_target_and_per_occurrence_dedup() {
    use super::scan_embedded_fragments;
    use crate::codebase::postgres::embedded::{
        extract_embedded_sql_from_source, CallSqlFragment, EmbeddedSqlOptions,
    };
    use crate::codebase::rules::PostgresFindings;
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::path::Path;
    use std::sync::Arc;

    let catalog = test_catalog();
    let facts = CheckFactMap::default();
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let mut file = extract_embedded_sql_from_source(
        Path::new("/repo/src/a.ts"),
        "",
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let mk = |sql: &str, line, span, ctx: Option<&str>, append_site| CallSqlFragment {
        sql: sql.to_string(),
        line,
        span,
        context_sql: ctx.map(str::to_string),
        append_site,
    };
    file.call_spans = vec![(0, 10), (11, 20), (21, 30), (31, 40), (41, 50), (51, 60)];
    for (span, frag) in [
        (
            (51, 60),
            mk(
                "SELECT * FROM wide; UPDATE wide SET id = 1 RETURNING *",
                5,
                (51, 60),
                None,
                None,
            ),
        ),
        (
            (41, 50),
            mk("wide SET a = 1 RETURNING *", 3, (41, 50), Some("UPDATE "), None),
        ),
        (
            (0, 10),
            mk(
                "RETURNING *",
                1,
                (0, 10),
                Some("WITH wide AS (SELECT 1) UPDATE wide SET a = 1"),
                None,
            ),
        ),
        (
            (11, 20),
            mk("SELECT row_to_json(w.*) FROM wide w", 1, (11, 20), None, None),
        ),
        (
            (21, 30),
            mk("SELECT * FROM unknown_tbl", 1, (21, 30), None, None),
        ),
        (
            (31, 40),
            mk("SELECT q.*", 2, (31, 40), Some("SELECT 1 FROM (SELECT 1) AS q"), None),
        ),
    ] {
        file.call_fragments.insert(span, vec![frag]);
    }

    let mut findings = PostgresFindings::default();
    scan_embedded_fragments(
        Path::new("/repo"),
        &test_options(),
        &[&file],
        &catalog,
        &sources,
        &facts,
        &mut findings,
    );
    assert_eq!(findings.0.len(), 4);
    assert_eq!(facts.sql_verdicts().len(), 1);

    let no_ret = compile_options(&Options {
        import_specifier: "@example/db".to_string(),
        schema_catalog_path: "db/schema.json".to_string(),
        relations: vec!["wide".to_string()],
        check_returning: Some(false),
        ..Default::default()
    })
    .unwrap();
    let mut findings2 = PostgresFindings::default();
    scan_embedded_fragments(
        Path::new("/repo"),
        &no_ret,
        &[&file],
        &catalog,
        &sources,
        &CheckFactMap::default(),
        &mut findings2,
    );
    assert_eq!(findings2.0.len(), 1);

    let mut dedup_file = extract_embedded_sql_from_source(
        Path::new("/repo/src/dedup.ts"),
        r#"import { query } from "@example/db";
import sql from "sql-template-strings";
export function f(flag: boolean) {
  const q = sql`SELECT 1`;
  if (flag) q.append(sql` WHERE id = 1`);
  query(q);
}"#,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    let covered = dedup_file.calls[0]
        .variants
        .iter()
        .flat_map(|v| v.append_sites.iter().copied())
        .next()
        .unwrap();
    dedup_file.call_spans = vec![(0, 5)];
    dedup_file.call_fragments.insert(
        (0, 5),
        vec![
            mk("SELECT * FROM wide", 1, (0, 5), None, None),
            mk("SELECT * FROM wide", 1, (0, 5), None, None),
            mk("SELECT missing.*", 2, (6, 10), None, None),
            mk("SELECT missing.*", 2, (6, 10), None, None),
        ],
    );
    dedup_file.param_append_fragments = vec![
        mk("SELECT * FROM wide", 3, (20, 30), None, Some(covered)),
        mk("SELECT * FROM wide", 4, (31, 40), None, Some(covered + 999)),
    ];
    let (mut findings3, facts3) = (PostgresFindings::default(), CheckFactMap::default());
    scan_embedded_fragments(
        Path::new("/repo"),
        &test_options(),
        &[&dedup_file],
        &catalog,
        &sources,
        &facts3,
        &mut findings3,
    );
    assert_eq!(findings3.0.len(), 2);
    assert_eq!(facts3.sql_verdicts().len(), 1);
}
