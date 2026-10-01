use super::super::{AllowEntry, CompiledOptions, RULE_ID};
use super::check::{self, exempt};
use crate::codebase::postgres::{
    CatalogTable, PartitionKeyElement, RelationKind, SchemaCatalog, SqlRelationPredicateFact,
};
use crate::codebase::rules::RuleFinding;
use std::collections::{BTreeSet, HashSet};

pub(super) fn catalog_findings(
    path: &str,
    catalog: &SchemaCatalog,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for table in catalog.tables() {
        if table.relation_kind != RelationKind::PartitionedTable
            || exempt(&table.name, &opts.partition_key_exemptions)
        {
            continue;
        }
        let Some(key) = &table.partition_key else {
            continue;
        };
        for element in &key.elements {
            let PartitionKeyElement::Expression(expression) = element else {
                continue;
            };
            let object = format!("table:{}", unqualified(&table.name));
            findings.push(catalog_finding(
                path,
                &object,
                &format!(
                    "cannot derive a column from partition key expression {expression}; add a partitionKeyExemptions entry for {}",
                    unqualified(&table.name)
                ),
            ));
        }
    }
    for entry in &opts.partition_key_exemptions {
        if !catalog.tables().any(|table| {
            table.relation_kind == RelationKind::PartitionedTable
                && check::table_matches(&table.name, &entry.table)
        }) {
            let object = format!("table:{}", unqualified(&entry.table));
            findings.push(catalog_finding(
                path,
                &object,
                &format!(
                    "stale {RULE_ID} partitionKeyExemptions entry: {}",
                    entry.table
                ),
            ));
        }
    }
    let expression_targets: HashSet<_> = findings
        .iter()
        .filter(|finding| finding.message.contains("cannot derive a column"))
        .filter_map(|finding| finding.target.clone())
        .collect();
    findings.retain(|finding| {
        finding.target.as_ref().is_none_or(|target| {
            !allowed(opts, target) || !finding.message.contains("cannot derive a column")
        })
    });
    findings.extend(stale_allows(&opts.allow, &expression_targets, path));
    findings
}

pub(super) fn partition_columns(
    file: &str,
    relation: &SqlRelationPredicateFact,
    word: &str,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
    columns: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    let Some(catalog) = catalog else {
        return Vec::new();
    };
    if exempt(&relation.table, &opts.partition_key_exemptions) {
        return Vec::new();
    }
    let Some(table) = catalog.relation(&relation.table) else {
        return Vec::new();
    };
    if table.relation_kind != RelationKind::PartitionedTable || has_expression(table) {
        return Vec::new();
    }
    let Some(key) = &table.partition_key else {
        return Vec::new();
    };
    key.elements.iter().filter_map(column_element)
        .filter(|required| !columns.iter().any(|column| column.eq_ignore_ascii_case(required)))
        .map(|required| {
            check::sql_finding(
                file,
                relation.line.max(1),
                &format!(
                    "{word} reads partitioned table {} without constraining partition key column {required}; add a predicate on {required} or add a partitionKeyExemptions entry",
                    relation.table
                ),
                Some(relation.table.as_str()),
                Some(check::import_for(relation, required)),
            )
        })
        .collect()
}

fn stale_allows(allow: &[AllowEntry], produced: &HashSet<String>, path: &str) -> Vec<RuleFinding> {
    allow
        .iter()
        .filter(|entry| !produced.contains(&entry.object))
        .map(|entry| {
            catalog_finding(
                path,
                &entry.object,
                &format!("stale {RULE_ID} allow entry: {}", entry.object),
            )
        })
        .collect()
}

fn allowed(opts: &CompiledOptions, target: &str) -> bool {
    opts.allow.iter().any(|entry| entry.object == target)
}

fn column_element(element: &PartitionKeyElement) -> Option<&str> {
    match element {
        PartitionKeyElement::Column(column) => Some(column.as_str()),
        PartitionKeyElement::Expression(_) => None,
    }
}

fn has_expression(table: &CatalogTable) -> bool {
    table.partition_key.as_ref().is_some_and(|key| {
        key.elements
            .iter()
            .any(|element| matches!(element, PartitionKeyElement::Expression(_)))
    })
}

fn unqualified(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

fn catalog_finding(path: &str, object: &str, text: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: path.to_string(),
        line: 1,
        message: format!("{path}: {object}: {text}"),
        import: None,
        target: Some(object.to_string()),
    }
}

#[cfg(test)]
mod element_tests {
    use super::column_element;
    use crate::codebase::postgres::PartitionKeyElement;

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
        use crate::codebase::postgres::SqlRelationPredicateFact;
        use std::collections::BTreeSet;

        let opts = super::super::super::compile_options(
            &serde_yaml::from_str("partitionKeys: require\nschemaCatalogPath: schema.json\n")
                .unwrap(),
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
}
