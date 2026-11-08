use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_json_writes, SqlLocationDemand,
};
use crate::codebase::postgres::SchemaCatalog;
use crate::codebase::rules::postgres_bounded_statements::evaluate::offenders;

fn root() -> std::path::PathBuf {
    super::fixture_root().join("../materialized-bodies/probes")
}

fn catalog() -> SchemaCatalog {
    let sources = crate::codebase::rules::source_store_for_files(&[root().join("schema.json")]);
    SchemaCatalog::load(&root(), "schema.json", &sources).unwrap()
}

#[test]
fn existence_and_correlated_scalar_probes_are_not_candidate_pools() {
    let sql = std::fs::read_to_string(root().join("probes.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        true,
        None,
        SqlLocationDemand::None,
        false,
        false,
        true,
    );
    assert!(!facts.parse_failed);
    assert!(!facts.bounds.is_empty());
    assert!(facts.bounds.iter().all(|fact| fact.body.is_none()));
    assert!(facts
        .bounds
        .iter()
        .all(|fact| offenders(fact, &catalog()).is_empty()));
}

#[test]
fn inherited_pools_keep_their_source_obligation_and_new_scans_stay_visible() {
    let sql = std::fs::read_to_string(root().join("pools.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        true,
        None,
        SqlLocationDemand::None,
        false,
        false,
        true,
    );
    assert!(!facts.parse_failed);
    let catalog = catalog();
    let bodies: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.body.is_some())
        .collect();
    assert_eq!(bodies.len(), 13);
    let outcomes: Vec<_> = bodies
        .iter()
        .map(|fact| {
            (
                fact.line,
                offenders(fact, &catalog)
                    .into_iter()
                    .map(|offender| offender.table)
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert_eq!(
        outcomes,
        vec![
            (1, vec!["source_rows".to_owned()]),
            (3, vec![]),
            (5, vec![]),
            (9, vec!["edge_rows".to_owned()]),
            (10, vec!["edge_rows".to_owned()]),
            (12, vec!["source_rows".to_owned()]),
            (14, vec!["edge_rows".to_owned()]),
            (19, vec!["source_rows".to_owned()]),
            (20, vec!["source_rows".to_owned()]),
            (23, vec!["source_rows".to_owned()]),
            (24, vec!["edge_rows".to_owned()]),
            (28, vec!["source_rows".to_owned()]),
            (29, vec![]),
        ]
    );
}

#[test]
fn body_projections_preserve_every_ordinary_statement_fact() {
    for name in ["probes.sql", "pools.sql", "collections.sql", "nested.sql"] {
        let sql = std::fs::read_to_string(root().join(name)).unwrap();
        let extract = |enabled| {
            extract_sql_statement_facts_with_json_writes(
                &sql,
                true,
                None,
                SqlLocationDemand::None,
                false,
                false,
                enabled,
            )
        };
        let off = extract(false);
        let mut on = extract(true);
        on.bounds.retain(|fact| fact.body.is_none());
        assert_eq!(on, off, "{name}");
    }
}

#[test]
fn collection_operators_keep_pool_bodies_but_exclude_scalar_probe_operands() {
    let sql = std::fs::read_to_string(root().join("collections.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        true,
        None,
        SqlLocationDemand::None,
        false,
        false,
        true,
    );
    assert!(!facts.parse_failed);
    let bodies: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.body.is_some())
        .collect();
    assert_eq!(bodies.len(), 4);
    let catalog = catalog();
    for (line, fact) in (1..=4).zip(bodies) {
        assert_eq!(fact.line, line);
        let names: Vec<_> = offenders(fact, &catalog)
            .into_iter()
            .map(|offender| offender.table)
            .collect();
        assert_eq!(names, ["edge_rows"]);
    }
}

#[test]
fn materialized_pools_inside_existence_probes_keep_their_own_obligation() {
    let sql = std::fs::read_to_string(root().join("nested.sql")).unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        true,
        None,
        SqlLocationDemand::None,
        false,
        false,
        true,
    );
    let bodies: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.body.is_some())
        .collect();
    assert_eq!(bodies.len(), 1);
    assert_eq!(bodies[0].line, 2);
    assert_eq!(
        bodies[0].body.as_deref(),
        Some("MATERIALIZED CTE nested_pool")
    );
    assert_eq!(offenders(bodies[0], &catalog()).len(), 1);
}
