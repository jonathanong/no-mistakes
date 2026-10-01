use super::super::{CompiledOptions, RelationOption};
use super::catalog;
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
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for select in &facts.selects {
        findings.extend(textual_require(file, select, opts));
        let word = if select.in_insert_select {
            "INSERT … SELECT"
        } else {
            "SELECT"
        };
        findings.extend(relation_findings(
            file,
            &select.relations,
            word,
            opts,
            catalog,
        ));
    }
    for group in &facts.updates {
        findings.extend(relation_findings(file, group, "UPDATE", opts, catalog));
    }
    for group in &facts.deletes {
        findings.extend(relation_findings(file, group, "DELETE", opts, catalog));
    }
    findings
}

fn relation_findings(
    file: &str,
    relations: &[SqlRelationPredicateFact],
    word: &str,
    opts: &CompiledOptions,
    catalog_ref: Option<&SchemaCatalog>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for (index, relation) in relations.iter().enumerate() {
        let columns = effective_columns(relations, index, catalog_ref);
        for configured in &opts.relations {
            if !table_matches(&relation.table, &configured.table) {
                continue;
            }
            findings.extend(missing_columns(file, relation, word, configured, &columns));
        }
        if opts.partition_keys {
            findings.extend(catalog::partition_columns(
                file,
                relation,
                word,
                opts,
                catalog_ref,
                &columns,
            ));
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
