use super::super::{CompiledOptions, RULE_ID};
use super::check::{self, exempt};
use crate::codebase::postgres::{
    catalog_finding, AllowList, CatalogObjectRef, PartitionKeyElement, RelationKind, SchemaCatalog,
    SqlRelationPredicateFact,
};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeSet;

pub(super) fn catalog_findings(
    path: &str,
    catalog: &SchemaCatalog,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let tables: Vec<_> = catalog
        .logical_tables()
        .filter(|table| table.relation_kind == RelationKind::PartitionedTable)
        .collect();
    let mut expressions = Vec::new();
    let mut findings = Vec::new();
    for table in &tables {
        if exempt(&table.name, &opts.partition_key_exemptions) {
            continue;
        }
        let object = CatalogObjectRef::Table(table.name.clone());
        let Some(key) = &table.partition_key else {
            findings.push(catalog_finding(
                RULE_ID,
                path,
                &object,
                "partitioned table is missing physicalPartition.key; refresh the schema catalog",
            ));
            continue;
        };
        let expression: Vec<_> = key
            .elements
            .iter()
            .filter_map(|element| match element {
                PartitionKeyElement::Expression(expression) => Some(expression.as_str()),
                _ => None,
            })
            .collect();
        if !expression.is_empty() {
            expressions.push(catalog_finding(RULE_ID, path, &object, &format!("cannot derive a column from partition key expression {}; add a partitionKeyExemptions entry for {}", expression.join(", "), table.name)));
        }
    }
    findings.extend(
        AllowList::compile(RULE_ID, opts.allow.clone())
            .expect("allow entries were validated")
            .apply(path, expressions),
    );
    // Qualified and unqualified membership is indexed once, without cloning the catalog per exemption.
    let names: BTreeSet<_> = tables
        .iter()
        .flat_map(|table| [table.name.as_str(), unqualified(&table.name)])
        .collect();
    for entry in &opts.partition_key_exemptions {
        if !names.contains(entry.table.as_str()) {
            findings.push(catalog_finding(
                RULE_ID,
                path,
                &CatalogObjectRef::Table(entry.table.clone()),
                &format!(
                    "stale {RULE_ID} partitionKeyExemptions entry: {}",
                    entry.table
                ),
            ));
        }
    }
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
    if table.relation_kind != RelationKind::PartitionedTable {
        return Vec::new();
    }
    let Some(key) = &table.partition_key else {
        return Vec::new();
    };
    key.elements.iter().filter_map(column_element)
        .filter(|required| !columns.contains(*required))
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

fn column_element(element: &PartitionKeyElement) -> Option<&str> {
    match element {
        PartitionKeyElement::Column(column) => Some(column.as_str()),
        PartitionKeyElement::Expression(_) => None,
    }
}

fn unqualified(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

#[cfg(test)]
mod tests;
