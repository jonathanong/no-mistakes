use super::super::super::{evaluate, ViewBounds};
use crate::codebase::postgres::statements::{
    SqlBoundItem, SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlPinSource,
};
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog, SqlBoundItemKind};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture")
}

fn catalog() -> SchemaCatalog {
    let root = fixture();
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

#[test]
fn recursive_finite_values_bound_keys_without_claiming_finite_recursive_rows() {
    let sql = std::fs::read_to_string(fixture().join("r3-domain-basic.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    let catalog = catalog();
    for (fact, expected) in facts.bounds.iter().zip([true, true, false, true, false]) {
        let found = evaluate(
            &fact.query,
            &catalog,
            &ViewBounds::default(),
            false,
            fact.target,
        );
        assert_eq!(found.items[0], expected);
    }
    for index in [0, 3] {
        let SqlBoundItemKind::Query(recursive) = &facts.bounds[index].query.items[1].kind else {
            panic!("expected recursive source");
        };
        let found = evaluate(recursive, &catalog, &ViewBounds::default(), false, None);
        assert!(!found.bounded);
        assert!(found
            .finite_outputs
            .as_ref()
            .is_some_and(|outputs| outputs[0]));
    }
}

#[test]
fn recursive_domain_guards_keep_unbounded_donors_and_transforms_unproved() {
    let sql = std::fs::read_to_string(fixture().join("r3-domain-guards.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 8);
    let catalog = catalog();
    for (fact, expected) in facts.bounds[..7]
        .iter()
        .zip([false, false, false, false, true, true, false])
    {
        let found = evaluate(
            &fact.query,
            &catalog,
            &ViewBounds::default(),
            false,
            fact.target,
        );
        assert_eq!(found.items[0], expected, "line {}", fact.line);
    }
    let SqlBoundItemKind::Query(recursive) = &facts.bounds[1].query.items[1].kind else {
        panic!("expected recursive source");
    };
    assert!(recursive.outputs[0].recursive_domain);
    let capped_anchor = evaluate(recursive, &catalog, &ViewBounds::default(), false, None);
    assert!(capped_anchor.items[0]);
    assert!(capped_anchor.finite_outputs.is_none());
    let selected = evaluate(
        &facts.bounds[7].query,
        &catalog,
        &ViewBounds::default(),
        false,
        None,
    );
    assert!(!selected.bounded);
}

#[test]
fn finite_domains_credit_write_targets_without_hiding_select_join_findings() {
    let sql = std::fs::read_to_string(fixture().join("r3-write-context.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    let catalog = catalog();
    let counts: Vec<_> = facts
        .bounds
        .iter()
        .map(|fact| super::super::super::offenders(fact, &catalog).len())
        .collect();
    assert_eq!(counts, [1, 0, 0, 0, 0]);
    let SqlBoundItemKind::Query(returning) = &facts.bounds[4].query.items[0].kind else {
        panic!("expected returning source");
    };
    let found = evaluate(returning, &catalog, &ViewBounds::default(), false, None);
    assert!(found.bounded);
}

#[test]
fn recursive_domains_require_a_current_with_donor_without_not_materialized() {
    let sql = std::fs::read_to_string(fixture().join("r3-materialization.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    let catalog = catalog();
    let counts: Vec<_> = facts
        .bounds
        .iter()
        .map(|fact| super::super::super::offenders(fact, &catalog).len())
        .collect();
    assert_eq!(counts, [1, 0, 0, 1, 1]);
}

#[test]
fn recursive_domain_defensive_paths_preserve_conservative_public_findings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/issue-3114-coverage/fixture");
    let sql = std::fs::read_to_string(root.join("recursive-domain-branches.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 11);
    let catalog = catalog();
    let expected = [0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0];
    for (fact, expected) in facts.bounds.iter().zip(expected) {
        assert_eq!(
            super::super::super::offenders(fact, &catalog).len(),
            expected,
            "line {}",
            fact.line,
        );
    }
}

fn output(name: Option<&str>, source: Option<(usize, &str)>, recursive: bool) -> SqlBoundOutput {
    SqlBoundOutput {
        name: name.map(str::to_owned),
        caller_sized: false,
        scalar: false,
        source_column: source.map(|(index, column)| (index, column.to_owned())),
        array_aggregate_column: None,
        constant_null: false,
        recursive_domain: recursive,
    }
}

fn query_item(outputs: Vec<SqlBoundOutput>) -> SqlBoundItem {
    SqlBoundItem::new(
        SqlBoundItemKind::Query(SqlBoundQuery {
            outputs,
            ..SqlBoundQuery::default()
        }),
        None,
        (1, 1),
    )
}

fn evaluation(finite: Vec<bool>) -> super::Evaluation {
    super::Evaluation {
        bounded: false,
        items: finite.clone(),
        finite_outputs: Some(finite),
        offenders: Vec::new(),
    }
}

#[test]
fn region_duplicate_exposed_names_are_not_finite_domains() {
    let query = SqlBoundQuery {
        items: vec![query_item(vec![
            output(Some("id"), None, false),
            output(Some("id"), None, false),
            output(Some("other"), None, false),
        ])],
        ..SqlBoundQuery::default()
    };
    let domains = super::prepare(&query, &[Some(evaluation(vec![true, true, true]))]);
    let names = domains
        .get(&0)
        .map(|names| names.iter().cloned().collect::<Vec<_>>());
    assert_eq!(names, Some(vec!["other".to_owned()]));
}

#[test]
fn region_recursive_domain_without_an_arm_is_not_finite() {
    let query = SqlBoundQuery {
        items: vec![
            query_item(vec![output(Some("id"), None, false)]),
            query_item(vec![output(Some("id"), None, false)]),
        ],
        outputs: vec![
            output(None, None, true),
            output(None, Some((1, "id")), false),
        ],
        ..SqlBoundQuery::default()
    };
    let nested = [None, Some(evaluation(vec![true]))];
    assert_eq!(
        super::output_domains(&query, &nested),
        Some(vec![false, true])
    );
}

#[test]
fn region_missing_domain_names_and_peer_lists_are_unproved() {
    let query = SqlBoundQuery {
        items: vec![query_item(vec![output(Some("id"), None, false)])],
        outputs: vec![
            output(None, Some((0, "missing")), false),
            output(None, Some((0, "id")), false),
        ],
        ..SqlBoundQuery::default()
    };
    assert_eq!(
        super::output_domains(&query, &[Some(evaluation(vec![true]))]),
        Some(vec![false, true])
    );
    assert!(super::exposed_name(&SqlBoundQuery::default(), 0, &[]).is_none());
    let unnamed = SqlBoundQuery {
        outputs: vec![output(None, None, true)],
        ..SqlBoundQuery::default()
    };
    assert!(super::exposed_name(&unnamed, 0, &[]).is_none());
    let table = SqlBoundQuery {
        items: vec![SqlBoundItem::new(
            SqlBoundItemKind::Table("accounts".into()),
            None,
            (1, 1),
        )],
        outputs: vec![output(None, None, true)],
        ..SqlBoundQuery::default()
    };
    assert!(super::exposed_name(&table, 0, &[]).is_none());
    let pin = SqlBoundPin {
        alternative: None,
        column: "id".into(),
        unqualified_candidate: false,
        expression: None,
        source: SqlPinSource::Items(vec![0, 1]),
        null_safe: false,
        reads: Vec::new(),
        qualified_reads: Vec::new(),
        peer_column: Some("id".into()),
    };
    let domains = super::Domains::default();
    assert!(!super::supplies(&pin, &domains));
}
