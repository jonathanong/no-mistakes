use super::{columns, CompiledOptions};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    record_sql_policy_verdicts, EmbeddedSqlFileFacts, PartitionKeyElement, RelationKind,
    SchemaCatalog, SqlBranchJudgment, SqlRelationPredicateFact, SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use std::path::Path;

enum Obligation<'a> {
    Text(&'a str, &'a str),
    Column(&'a str, &'a str),
    PartitionKey,
}

pub(super) fn record(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &[&EmbeddedSqlFileFacts],
    catalog: Option<&SchemaCatalog>,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<()> {
    let prepared = facts.prepared();
    let mut obligations = Vec::new();
    for relation in &opts.relations {
        for required in &relation.require {
            obligations.push((
                format!("relation:{}:predicate:{required}", relation.table),
                Obligation::Text(&relation.table, required.as_str()),
            ));
        }
        for required in &relation.require_columns {
            obligations.push((
                format!("relation:{}:column:{required}", relation.table),
                Obligation::Column(&relation.table, required.as_str()),
            ));
        }
    }
    if opts.partition_keys {
        obligations.push(("partition-key".to_string(), Obligation::PartitionKey));
    }
    let checks = obligations
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    for file in embedded {
        record_sql_policy_verdicts(
            crate::codebase::postgres::SqlPolicyVerdictContext {
                facts,
                embedded: file,
                statements: prepared.postgres_statements(&file.path, Some(&opts.embedded))?,
            },
            super::super::RULE_ID,
            &relative_slash_path(root, &file.path),
            &opts.embedded,
            &checks,
            opts.advise_unanalyzable,
            |check, _, statement| {
                let obligation = &obligations
                    .iter()
                    .find(|(name, _)| name == check)
                    .expect("enabled obligation is present")
                    .1;
                judge(obligation, statement, opts, catalog)
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests;

fn judge(
    obligation: &Obligation<'_>,
    statement: &SqlStatementFileFacts,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
) -> SqlBranchJudgment {
    match obligation {
        Obligation::Text(table, required) => {
            if statement.selects.iter().any(|select| {
                select
                    .tables
                    .iter()
                    .any(|found| found.eq_ignore_ascii_case(table))
                    && !columns::contains_predicate(&select.predicate_sql, required)
            }) {
                SqlBranchJudgment::violation(false)
            } else {
                SqlBranchJudgment::clear()
            }
        }
        Obligation::Column(table, required) => {
            let groups = relation_groups(statement);
            let mut matched = false;
            for relations in groups {
                for (index, relation) in relations.iter().enumerate() {
                    if !columns::table_matches(&relation.table, table) {
                        continue;
                    }
                    matched = true;
                    let Some(catalog) = catalog else {
                        return SqlBranchJudgment::abstain("missing-relation-catalog");
                    };
                    if catalog.relation(&relation.table).is_none() {
                        return SqlBranchJudgment::abstain("unknown-relation");
                    }
                    let effective = columns::effective_columns(relations, index, Some(catalog));
                    if !effective
                        .iter()
                        .any(|column| column.eq_ignore_ascii_case(required))
                    {
                        if unknown_peer(relations, index, catalog) {
                            return SqlBranchJudgment::abstain("unknown-relation-owner");
                        }
                        return SqlBranchJudgment::violation(false);
                    }
                }
            }
            if matched && catalog.is_none() {
                SqlBranchJudgment::abstain("missing-relation-catalog")
            } else {
                SqlBranchJudgment::clear()
            }
        }
        Obligation::PartitionKey => {
            let Some(catalog) = catalog else {
                return SqlBranchJudgment::abstain("missing-relation-catalog");
            };
            for relations in relation_groups(statement) {
                for (index, relation) in relations.iter().enumerate() {
                    if columns::exempt(&relation.table, &opts.partition_key_exemptions) {
                        continue;
                    }
                    let Some(info) = catalog.relation(&relation.table) else {
                        return SqlBranchJudgment::abstain("unknown-relation");
                    };
                    if info.relation_kind != RelationKind::PartitionedTable {
                        continue;
                    }
                    let Some(key) = &info.partition_key else {
                        return SqlBranchJudgment::abstain("missing-partition-key-catalog");
                    };
                    if key
                        .elements
                        .iter()
                        .any(|element| matches!(element, PartitionKeyElement::Expression(_)))
                    {
                        return SqlBranchJudgment::abstain("expression-partition-key");
                    }
                    let effective = columns::effective_columns(relations, index, Some(catalog));
                    if key.elements.iter().any(|element| match element {
                        PartitionKeyElement::Column(column) => !effective.contains(column),
                        PartitionKeyElement::Expression(_) => false,
                    }) {
                        if unknown_peer(relations, index, catalog) {
                            return SqlBranchJudgment::abstain("unknown-relation-owner");
                        }
                        return SqlBranchJudgment::violation(false);
                    }
                }
            }
            SqlBranchJudgment::clear()
        }
    }
}

fn unknown_peer(
    relations: &[SqlRelationPredicateFact],
    index: usize,
    catalog: &SchemaCatalog,
) -> bool {
    relations
        .iter()
        .enumerate()
        .any(|(other, peer)| other != index && catalog.relation(&peer.table).is_none())
}

fn relation_groups(statement: &SqlStatementFileFacts) -> Vec<&[SqlRelationPredicateFact]> {
    statement
        .selects
        .iter()
        .map(|select| select.relations.as_slice())
        .chain(statement.updates.iter().map(Vec::as_slice))
        .chain(statement.deletes.iter().map(Vec::as_slice))
        .collect()
}
