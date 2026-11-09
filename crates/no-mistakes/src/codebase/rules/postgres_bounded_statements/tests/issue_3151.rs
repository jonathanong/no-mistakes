use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn same_closed_range_bounds_complete_composite_keys_without_accepting_unsafe_ranges() {
    let root = super::fixture_root().join("../issue-3151/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    for (name, count) in [
        ("closed-parameter.sql", 0),
        ("closed-literal.sql", 0),
        ("closed-reversed.sql", 0),
        ("closed-or-branches.sql", 0),
        ("closed-parenthesized.sql", 0),
        ("closed-negative.sql", 0),
        ("closed-string.sql", 0),
        ("cross-or-control.sql", 1),
        ("cross-column-control.sql", 1),
        ("different-parameters.sql", 1),
        ("different-literals.sql", 1),
        ("open-range.sql", 1),
        ("or-range.sql", 1),
        ("cast-mismatch.sql", 1),
        ("volatile-range.sql", 1),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        assert!(!facts.parse_failed, "{name}");
        assert_eq!(facts.bounds.len(), 1, "{name}");
        assert_eq!(
            super::super::evaluate::offenders(&facts.bounds[0], &catalog).len(),
            count,
            "{name}"
        );
    }
}

#[test]
fn scalar_reads_are_ordinary_select_obligations_independent_of_pool_policy() {
    use crate::codebase::postgres::statements::{
        extract_sql_statement_facts_with_json_writes, SqlBoundKind, SqlLocationDemand,
    };
    let root = super::fixture_root().join("../issue-3151/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    for (name, expected) in [
        ("scalar-select.sql", 0),
        ("scalar-coalesce.sql", 0),
        ("scalar-closed.sql", 0),
        ("scalar-different-parameters.sql", 1),
        ("unbounded-scalar.sql", 1),
        ("unbounded-coalesce.sql", 1),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        for materialized in [false, true] {
            let facts = extract_sql_statement_facts_with_json_writes(
                &sql,
                true,
                None,
                SqlLocationDemand::None,
                false,
                false,
                materialized,
            );
            assert!(!facts.parse_failed, "{name}");
            assert_eq!(facts.bounds.len(), 2, "{name}");
            assert!(facts
                .bounds
                .iter()
                .all(|fact| fact.body.is_none() && fact.kind == SqlBoundKind::Select));
            let count: usize = facts
                .bounds
                .iter()
                .map(|fact| super::super::evaluate::offenders(fact, &catalog).len())
                .sum();
            assert_eq!(count, expected, "{name}, pools={materialized}");
        }
    }
}

#[test]
fn dml_scalar_reads_keep_statement_selection_and_read_only_obligations() {
    use crate::codebase::postgres::statements::{
        extract_sql_statement_facts_with_json_writes, SqlBoundKind, SqlLocationDemand,
    };
    let root = super::fixture_root().join("../issue-3151/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let sql = std::fs::read_to_string(root.join("dml-scalars.sql")).unwrap();
    for materialized in [false, true] {
        let facts = extract_sql_statement_facts_with_json_writes(
            &sql,
            true,
            None,
            SqlLocationDemand::None,
            false,
            false,
            materialized,
        );
        assert!(!facts.parse_failed);
        assert_eq!(facts.bounds.len(), 4);
        for (kind, expected) in [(SqlBoundKind::Update, 2), (SqlBoundKind::Delete, 2)] {
            let selected: Vec<_> = facts
                .bounds
                .iter()
                .filter(|fact| fact.kind == kind)
                .collect();
            assert_eq!(selected.len(), expected);
            assert!(selected.iter().all(|fact| fact.body.is_none()));
            let scalar = selected.iter().find(|fact| fact.target.is_none()).unwrap();
            assert_eq!(super::super::evaluate::offenders(scalar, &catalog).len(), 1);
            let write = selected.iter().find(|fact| fact.target.is_some()).unwrap();
            assert!(super::super::evaluate::offenders(write, &catalog).is_empty());
        }
    }
}

#[test]
fn scalar_scope_and_partial_predicate_controls_keep_outer_bounds() {
    use crate::codebase::postgres::statements::{
        extract_sql_statement_facts_with_json_writes, SqlLocationDemand,
    };
    let root = super::fixture_root().join("../issue-3151-scopes/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    for (name, expected) in [
        ("correlated-unqualified.sql", 0),
        ("correlated-lateral.sql", 0),
        ("partial-closed-range.sql", 0),
        ("partial-coalesce-and-exists.sql", 0),
        ("lateral-enclosing-composite.sql", 0),
        ("correlated-cte-limit.sql", 0),
        ("correlated-lateral-cte-limit.sql", 0),
        ("partial-nested-predicate.sql", 0),
        ("partial-quoted-predicate.sql", 0),
        ("correlated-derived-column.sql", 0),
        ("correlated-base-aliases.sql", 0),
        ("correlated-partial-base-aliases.sql", 0),
        ("correlated-unnest.sql", 0),
        ("partial-mixed-case-control.sql", 1),
        ("partial-mixed-key-control.sql", 1),
        ("partial-mixed-key-lookup.sql", 0),
        ("partial-mixed-predicate-lookup.sql", 0),
        ("unknown-outer-control.sql", 1),
        ("unknown-inner-control.sql", 1),
        ("temporary-inner-control.sql", 1),
        ("temporary-outer-control.sql", 1),
        ("uncapped-cte-control.sql", 1),
        ("partial-missing-predicate.sql", 1),
        ("partial-wrong-predicate.sql", 1),
        ("unbounded-scalar.sql", 1),
        ("unbounded-lateral.sql", 1),
        ("unbounded-shadowed-column.sql", 1),
        ("correlated-nonunique-control.sql", 1),
        ("correlated-local-alias-control.sql", 1),
        ("correlated-or-control.sql", 1),
        ("correlated-volatile-control.sql", 1),
        ("correlated-redundant-key.sql", 0),
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        for materialized in [false, true] {
            let facts = extract_sql_statement_facts_with_json_writes(
                &sql,
                true,
                None,
                SqlLocationDemand::None,
                false,
                false,
                materialized,
            );
            assert!(!facts.parse_failed, "{name}");
            assert!(!facts.bounds.is_empty(), "{name}: missing statement bounds");
            let count: usize = facts
                .bounds
                .iter()
                .map(|fact| super::super::evaluate::offenders(fact, &catalog).len())
                .sum();
            assert_eq!(count, expected, "{name}, materialized={materialized}");
        }
    }
}

#[test]
fn nested_lateral_controls_keep_incomplete_or_local_key_sources() {
    let root = super::fixture_root().join("../issue-3151-scopes/fixture");
    let sources = crate::codebase::rules::source_store_for_files(&[root.join("schema.json")]);
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    for name in [
        "lateral-missing-composite-half.sql",
        "lateral-shadowed-enclosing-alias.sql",
        "nonlateral-sibling-control.sql",
        "lateral-unbounded-driver.sql",
        "lateral-nonunique-control.sql",
        "lateral-local-self-control.sql",
    ] {
        let sql = std::fs::read_to_string(root.join(name)).unwrap();
        let facts =
            crate::codebase::postgres::statements::extract_sql_statement_facts_with_json_writes(
                &sql,
                true,
                None,
                crate::codebase::postgres::statements::SqlLocationDemand::None,
                false,
                false,
                true,
            );
        assert!(!facts.parse_failed, "{name}");
        let offenders: Vec<_> = facts
            .bounds
            .iter()
            .filter(|fact| !super::super::evaluate::offenders(fact, &catalog).is_empty())
            .collect();
        assert!(!offenders.is_empty(), "{name}");
        if name != "lateral-unbounded-driver.sql" {
            assert!(
                offenders.iter().any(|fact| fact.body.is_some()),
                "{name}: missing body obligation"
            );
        }
    }
}
