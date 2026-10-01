use super::column_element;
use crate::codebase::postgres::{PartitionKeyElement, SqlRelationPredicateFact};
use std::collections::BTreeSet;

#[test]
fn expression_partition_elements_are_not_required_columns() {
    assert_eq!(
        column_element(&PartitionKeyElement::Expression(
            "date_trunc('day', created_at)".into()
        )),
        None
    );
    assert_eq!(
        column_element(&PartitionKeyElement::Column("account_id".into())),
        Some("account_id")
    );
}

#[test]
fn a_missing_catalog_adds_no_partition_findings() {
    let opts = super::super::super::compile_options(
        &serde_yaml::from_str("partitionKeys: require\nschemaCatalogPath: schema.json\n").unwrap(),
    )
    .unwrap();
    let relation = SqlRelationPredicateFact {
        table: "events".into(),
        alias: None,
        constrained_columns: Vec::new(),
        unqualified_columns: Vec::new(),
        line: 1,
    };
    assert!(super::partition_columns(
        "query.sql",
        &relation,
        "SELECT",
        &opts,
        None,
        &BTreeSet::new(),
    )
    .is_empty());
}
