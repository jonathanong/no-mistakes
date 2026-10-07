use super::super::{ddl, locations::Locations};
use crate::codebase::postgres::{parse::prepare_postgres_tokens, statements::TableTokenIndex};
use sqlparser::{ast::Statement, dialect::PostgreSqlDialect, parser::Parser};

fn fixture() -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/ddl.sql"),
    )
    .unwrap()
}

#[test]
fn view_dependencies_follow_cte_definition_order_and_quoted_identity() {
    let sql = fixture();
    let locations = Locations::new(&sql);
    let mut prepared = prepare_postgres_tokens(&sql);
    ddl::prepare_trigger_arguments(&mut prepared.tokens);
    let tokens = TableTokenIndex::from_iter(&prepared.tokens);
    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens)
        .parse_statements()
        .unwrap();
    let expected = [
        vec!["accounts", "app.orders"],
        vec!["\"App\".\"Orders\""],
        vec![],
        vec!["base_rows", "more_rows"],
        vec!["branches", "roots"],
    ];
    for (statement, expected) in statements.iter().zip(expected) {
        let Statement::CreateView(view) = statement else {
            panic!("expected view")
        };
        let facts = ddl::view(view, &locations, &tokens);
        assert_eq!(
            facts
                .dependencies
                .iter()
                .map(|name| name.sql.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let Statement::CreateView(view) = &statements[0] else {
        unreachable!()
    };
    let facts = ddl::view(view, &locations, &tokens);
    assert!(facts.columns[0].quoted);
    assert_eq!(facts.columns[0].identity, "Total");
    let Statement::CreateView(view) = &statements[1] else {
        unreachable!()
    };
    assert!(ddl::view(view, &locations, &tokens).materialized);
    let Statement::CreateView(view) = &statements[2] else {
        unreachable!()
    };
    assert!(ddl::view(view, &locations, &tokens).temporary);
}

#[test]
fn function_facts_retain_signature_defaults_body_and_execution_flags() {
    let sql = fixture();
    let locations = Locations::new(&sql);
    let mut prepared = prepare_postgres_tokens(&sql);
    ddl::prepare_trigger_arguments(&mut prepared.tokens);

    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens)
        .parse_statements()
        .unwrap();
    let Statement::CreateFunction(function) = &statements[5] else {
        panic!("expected function")
    };
    let facts = ddl::function(function, &locations);
    assert_eq!(facts.name.sql, "app.adjust");
    assert!(facts.or_replace);
    assert!(!facts.temporary);
    assert_eq!(facts.arguments.len(), 2);
    assert_eq!(facts.arguments[1].mode.as_deref(), Some("INOUT"));
    assert_eq!(facts.arguments[1].default.as_ref().unwrap().sql, "'x'");
    assert_eq!(facts.language.as_deref(), Some("sql"));
    assert_eq!(facts.behavior.as_deref(), Some("IMMUTABLE"));
    assert_eq!(facts.called_on_null.as_deref(), Some("STRICT"));
    assert_eq!(facts.parallel.as_deref(), Some("PARALLEL SAFE"));
    assert_eq!(facts.security.as_deref(), Some("SECURITY INVOKER"));
    assert_eq!(facts.configuration.len(), 1);
    assert!(facts.body_sql.as_ref().unwrap().contains("SELECT label"));
    assert!(facts.return_type.is_some());
    assert!(!facts.returns_set);
    let Statement::CreateFunction(function) = &statements[6] else {
        unreachable!()
    };
    let facts = ddl::function(function, &locations);
    assert!(facts.returns_set);
    assert!(facts.return_type.unwrap().sql.contains("app.\"Row\""));
}

#[test]
fn trigger_facts_retain_transition_relations_conditions_and_references() {
    let sql = fixture();
    let locations = Locations::new(&sql);
    let mut prepared = prepare_postgres_tokens(&sql);
    ddl::prepare_trigger_arguments(&mut prepared.tokens);

    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens)
        .parse_statements()
        .unwrap();
    let Statement::CreateTrigger(trigger) = &statements[8] else {
        panic!("expected trigger")
    };
    let facts = ddl::trigger(trigger, &locations);
    assert!(facts.or_replace);
    assert_eq!(facts.function.unwrap().sql, "app.audit");
    assert_eq!(facts.events, ["INSERT"]);
    assert_eq!(facts.transitions[0].name.sql, "new_rows");
    assert_eq!(facts.transitions[0].kind, "NEW TABLE");
    assert_eq!(facts.execution_kind.as_deref(), Some("FUNCTION"));
    assert_eq!(facts.for_each.as_deref(), Some("FOR EACH STATEMENT"));
    let Statement::CreateTrigger(trigger) = &statements[9] else {
        unreachable!()
    };
    let facts = ddl::trigger(trigger, &locations);
    assert!(facts.constraint);
    assert_eq!(facts.referenced_table.unwrap().sql, "app.parents");
    assert!(facts
        .characteristics
        .unwrap()
        .contains("INITIALLY DEFERRED"));
    let condition = facts.condition.unwrap();
    assert!(condition.span.is_some());
    assert_eq!(condition.columns.len(), 2);
    assert_eq!(facts.execution_kind.as_deref(), Some("PROCEDURE"));
}

#[test]
fn dependency_facts_deduplicate_and_ignore_function_and_forward_cte_names() {
    let sql = fixture();
    let locations = Locations::new(&sql);
    let mut prepared = prepare_postgres_tokens(&sql);
    ddl::prepare_trigger_arguments(&mut prepared.tokens);
    let tokens = TableTokenIndex::from_iter(&prepared.tokens);
    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens)
        .parse_statements()
        .unwrap();
    for (index, expected) in [
        (10, vec![]),
        (11, vec!["app.orders"]),
        (12, vec!["physical_rows"]),
    ] {
        let Statement::CreateView(view) = &statements[index] else {
            panic!("expected view")
        };
        assert_eq!(
            ddl::view(view, &locations, &tokens)
                .dependencies
                .iter()
                .map(|name| name.sql.clone())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let Statement::CreateFunction(function) = &statements[13] else {
        panic!("expected function")
    };
    let facts = ddl::function(function, &locations);
    assert!(facts.return_type.is_none());
    assert!(!facts.returns_set);
    let Statement::CreateTrigger(trigger) = &statements[14] else {
        panic!("expected trigger")
    };
    let facts = ddl::trigger(trigger, &locations);
    assert_eq!(
        facts.arguments,
        [
            "'it''s'",
            "42",
            "'null'",
            "'integer'",
            "'integer'",
            "'mixed'",
            "'MiXeD'"
        ]
    );
    assert_eq!(facts.events, ["INSERT", "DELETE"]);
}

#[test]
fn table_view_dependencies_reuse_prepared_identity_and_report_ambiguity() {
    use super::super::{parse_postgres_source, PostgresSqlSource, PostgresSqlStatementKind};
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/view-table.sql"),
    )
    .unwrap();
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql,
        file_name: None,
    });
    assert_eq!(facts.statements.len(), 4, "{:?}", facts.diagnostics);
    for (index, expected) in [
        (0, "\"App\".\"Orders\""),
        (1, "app.orders"),
        (2, "app.orders"),
    ] {
        let PostgresSqlStatementKind::CreateView { view } = &facts.statements[index].facts else {
            panic!("expected view")
        };
        assert!(view.dependencies_complete);
        assert_eq!(
            view.dependencies
                .iter()
                .map(|name| name.sql.as_str())
                .collect::<Vec<_>>(),
            [expected]
        );
    }
    let PostgresSqlStatementKind::CreateView { view } = &facts.statements[3].facts else {
        panic!("expected view")
    };
    assert!(!view.dependencies_complete);
    assert!(view.dependencies.is_empty());
    assert_eq!(facts.diagnostics.len(), 1);
}

#[test]
fn typed_function_body_projection_covers_supported_parser_variants() {
    use sqlparser::dialect::{BigQueryDialect, Dialect, MsSqlDialect};
    // The typed projection accepts the parser enum; other dialect fixtures protect
    // defensive variants without claiming PostgreSQL source grammar supports them.
    let cases: [(&str, &dyn Dialect, usize); 3] = [
        ("function-body-bigquery.sql", &BigQueryDialect {}, 1),
        ("function-body-mssql.sql", &MsSqlDialect {}, 3),
        ("function-body-postgres.sql", &PostgreSqlDialect {}, 2),
    ];
    for (file, dialect, expected) in cases {
        let sql = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/postgres-facts/source")
                .join(file),
        )
        .unwrap();
        let locations = Locations::new(&sql);
        let statements = Parser::parse_sql(dialect, &sql).unwrap();
        assert_eq!(statements.len(), expected);
        for statement in statements {
            let Statement::CreateFunction(function) = statement else {
                panic!("expected function")
            };
            let facts = ddl::function(&function, &locations);
            assert!(facts.body_sql.is_some());
        }
    }
}

#[test]
fn malformed_trigger_inputs_return_diagnostics_and_preserve_neighboring_statements() {
    use super::super::{parse_postgres_source, PostgresSqlSource};
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/trigger-invalid.sql"),
    )
    .unwrap();
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql,
        file_name: None,
    });
    assert!(facts.diagnostics.len() >= 5);
    assert!(facts.statements.first().unwrap().sql.contains("SELECT 1"));
    assert!(facts.statements.last().unwrap().sql.contains("SELECT 2"));
}

#[test]
fn view_function_references_share_dependency_walk_and_keep_each_source_span() {
    use super::super::{parse_postgres_source, PostgresSqlSource, PostgresSqlStatementKind};
    let sql = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source/view-functions.sql"),
    )
    .unwrap();
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: sql.clone(),
        file_name: None,
    });
    assert!(facts.diagnostics.is_empty());
    let PostgresSqlStatementKind::CreateView { view } = &facts.statements[0].facts else {
        panic!("expected view")
    };
    assert_eq!(view.dependencies[0].sql, "app.accounts");
    assert_eq!(
        view.functions
            .iter()
            .map(|reference| reference.name.sql.as_str())
            .collect::<Vec<_>>(),
        [
            "app.allowed",
            "app.allowed",
            "app.decorate",
            "pg_catalog.lower"
        ]
    );
    for reference in &view.functions {
        let span = reference.span.as_ref().unwrap();
        assert!(sql[span.start.offset..span.end.offset].starts_with(&reference.name.sql));
    }
}

#[test]
fn table_identity_rejects_other_candidates_and_accepts_an_empty_inventory() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/source");
    let empty = std::fs::read_to_string(root.join("empty.sql")).unwrap();
    let prepared = prepare_postgres_tokens(&empty);
    let index = TableTokenIndex::from_iter(&prepared.tokens);
    let sql = std::fs::read_to_string(root.join("view-table-candidates.sql")).unwrap();
    let prepared = prepare_postgres_tokens(&sql);
    let candidates = TableTokenIndex::from_iter(&prepared.tokens);
    let statements = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(prepared.tokens)
        .parse_statements()
        .unwrap();
    let Statement::CreateView(view) = &statements[0] else {
        panic!("expected view")
    };
    let locations = Locations::new(&sql);
    assert!(!ddl::view(view, &locations, &index).dependencies_complete);
    let facts = ddl::view(view, &locations, &candidates);
    assert!(facts.dependencies_complete);
    assert_eq!(
        facts
            .dependencies
            .iter()
            .map(|name| name.sql.as_str())
            .collect::<Vec<_>>(),
        ["app.customers", "app.orders", "other.orders"]
    );
}

#[test]
fn scalar_table_views_retain_source_identity_and_original_spans() {
    use super::super::PostgresSqlStatementKind;
    let sql = super::fixture("scalar-table-views.sql");
    let facts = super::facts("scalar-table-views.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 3);
    for (statement, expected) in
        facts
            .statements
            .iter()
            .zip([vec!["\"Helper\""], vec!["pg_temp.\"Mixed.Helper\""], vec![]])
    {
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
        let PostgresSqlStatementKind::CreateView { view } = &statement.facts else {
            panic!("expected view")
        };
        assert!(view.dependencies_complete);
        assert_eq!(
            view.dependencies
                .iter()
                .map(|name| name.sql.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn recursive_views_emit_typed_facts_and_preserve_ordinary_view_behavior() {
    use super::super::PostgresSqlStatementKind;
    let sql = super::fixture("recursive-views.sql");
    // Both native parser paths must accept the same saved declarations.
    let statements = crate::codebase::postgres::parse_postgres_sql(&sql).unwrap();
    assert_eq!(statements.len(), 8);
    assert!(statements[..7]
        .iter()
        .all(|statement| matches!(statement, Statement::CreateView(_))));
    let facts = super::facts("recursive-views.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 8);
    for (index, expected, quoted, replace, materialized) in [
        (0, vec!["recursive_view"], false, false, false),
        (1, vec!["App", "RecursiveView"], true, true, false),
        (2, vec!["app", "commented"], false, false, false),
        (3, vec!["ordinary_view"], false, false, false),
        (4, vec!["App", "OrdinaryView"], true, true, false),
        (5, vec!["materialized_view"], false, false, true),
        (6, vec!["recursive"], true, false, false),
    ] {
        let statement = &facts.statements[index];
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
        let PostgresSqlStatementKind::CreateView { view } = &statement.facts else {
            panic!("expected createView")
        };
        assert_eq!(
            view.name
                .parts
                .iter()
                .map(|part| part.identity.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(view.name.parts.iter().all(|part| part.quoted == quoted));
        assert_eq!(view.materialized, materialized);
        assert_eq!(view.or_replace, replace);
        assert_eq!(view.columns.len(), 1);
        assert_eq!(
            view.columns[0].identity,
            if index == 1 || index == 4 { "X" } else { "x" }
        );
        assert!(view.dependencies_complete);
    }
    let view = |index: usize| {
        let PostgresSqlStatementKind::CreateView { view } = &facts.statements[index].facts else {
            panic!("expected createView")
        };
        view
    };
    assert_eq!(view(0).query, view(3).query);
    assert_eq!(view(1).query, view(4).query);
    assert_eq!(view(0).dependencies, view(3).dependencies);
    assert_eq!(view(1).dependencies, view(4).dependencies);
    assert!(view(6).query.starts_with("WITH RECURSIVE"));
    assert!(view(6).dependencies.is_empty());
}

#[test]
fn invalid_recursive_declarations_keep_diagnostics_and_valid_neighbors() {
    let facts = super::facts("recursive-views-invalid.sql");
    assert_eq!(facts.diagnostics.len(), 4);
    assert_eq!(facts.statements.len(), 1);
    assert_eq!(facts.statements[0].ordinal, 4);
    assert_eq!(facts.statements[0].sql, "SELECT 2;");
}
