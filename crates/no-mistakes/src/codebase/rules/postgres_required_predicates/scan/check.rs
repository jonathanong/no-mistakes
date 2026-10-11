use super::super::{CompiledOptions, RelationOption};
use super::catalog;
use crate::codebase::postgres::statements::SqlFactSite;
use crate::codebase::postgres::{SchemaCatalog, SqlRelationPredicateFact, SqlStatementFileFacts};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeSet;

pub(super) use super::columns::{
    effective_columns, exempt, import_for, sql_finding, table_matches, textual_require,
};

pub(super) fn statement_findings(
    file: &str,
    facts: &SqlStatementFileFacts,
    opts: &CompiledOptions,
    catalog: Option<&SchemaCatalog>,
) -> Vec<(SqlFactSite, RuleFinding)> {
    let mut findings = Vec::new();
    for (select_index, select) in facts.selects.iter().enumerate() {
        findings.extend(
            textual_require(file, select, opts)
                .into_iter()
                .map(|finding| (SqlFactSite::Select(select_index), finding)),
        );
        let word = if select.in_insert_select {
            "INSERT … SELECT"
        } else {
            "SELECT"
        };
        findings.extend(
            relation_findings(file, &select.relations, word, opts, catalog)
                .into_iter()
                .map(|(relation_index, finding)| {
                    (SqlFactSite::Relation(select_index, relation_index), finding)
                }),
        );
    }
    for (group_index, group) in facts.updates.iter().enumerate() {
        findings.extend(
            relation_findings(file, group, "UPDATE", opts, catalog)
                .into_iter()
                .map(|(index, finding)| (SqlFactSite::UpdateRelation(group_index, index), finding)),
        );
    }
    for (group_index, group) in facts.deletes.iter().enumerate() {
        findings.extend(
            relation_findings(file, group, "DELETE", opts, catalog)
                .into_iter()
                .map(|(index, finding)| (SqlFactSite::DeleteRelation(group_index, index), finding)),
        );
    }
    findings
}

fn relation_findings(
    file: &str,
    relations: &[SqlRelationPredicateFact],
    word: &str,
    opts: &CompiledOptions,
    catalog_ref: Option<&SchemaCatalog>,
) -> Vec<(usize, RuleFinding)> {
    let mut findings = Vec::new();
    for (index, relation) in relations.iter().enumerate() {
        let columns = effective_columns(relations, index, catalog_ref);
        for configured in &opts.relations {
            if !table_matches(&relation.table, &configured.table) {
                continue;
            }
            findings.extend(
                missing_columns(file, relation, word, configured, &columns)
                    .into_iter()
                    .map(|finding| (index, finding)),
            );
        }
        if opts.partition_keys {
            findings.extend(
                catalog::partition_columns(file, relation, word, opts, catalog_ref, &columns)
                    .into_iter()
                    .map(|finding| (index, finding)),
            );
        }
    }
    findings
}

fn missing_columns(
    file: &str,
    relation: &SqlRelationPredicateFact,
    word: &str,
    configured: &RelationOption,
    columns: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    configured
        .require_columns
        .iter()
        .filter(|required| {
            !columns
                .iter()
                .any(|column| column.eq_ignore_ascii_case(required))
        })
        .map(|required| {
            sql_finding(
                file,
                relation.line.max(1),
                &format!(
                    "{word} on {} does not constrain required column {required}",
                    relation.table
                ),
                Some(relation.table.as_str()),
                Some(import_for(relation, required)),
            )
        })
        .collect()
}
