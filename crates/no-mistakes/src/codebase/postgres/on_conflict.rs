//! ON CONFLICT replay-safety judgments over statement facts.

mod arbiter;
mod convergence;
mod generated;
mod installed;
pub(crate) use installed::InstalledTriggers;
mod trigger;
mod trigger_writes;
mod verdicts;
mod where_noop;
pub(crate) use verdicts::judge_subcheck;

#[cfg(test)]
mod coverage_more_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod expression_arbiter_tests;
#[cfg(test)]
mod installed_tests;
#[cfg(test)]
mod trigger_tests;
#[cfg(test)]
mod where_noop_gap_tests;
#[cfg(test)]
mod where_noop_tests;

use crate::codebase::postgres::statement_facts::{
    SqlFactSite, SqlInsertFact, SqlOnConflictAction, SqlStatementFileFacts, SqlTriggerFact,
    SqlValueForm,
};
use crate::codebase::postgres::types::SqlSchemaFileFacts;

#[derive(Copy, Clone)]
pub struct Catalog<'a> {
    pub schema: &'a [SqlSchemaFileFacts],
    pub triggers: &'a [SqlTriggerFact],
    pub replay_safe: &'a [String],
    pub check_convergence: bool,
    pub check_volatility: bool,
    pub check_arbiter: bool,
    pub check_triggers: bool,
    pub check_generated: bool,
    pub trigger_writes: &'a [(String, Vec<String>)],
}

pub fn judge_file(file: &SqlStatementFileFacts, catalog: &Catalog<'_>) -> Vec<(usize, String)> {
    judge_file_with_sites(file, catalog)
        .into_iter()
        .map(|(_, line, message)| (line, message))
        .collect()
}

pub(crate) fn judge_file_with_sites(
    file: &SqlStatementFileFacts,
    catalog: &Catalog<'_>,
) -> Vec<(SqlFactSite, usize, String)> {
    let installed = catalog
        .check_triggers
        .then(|| InstalledTriggers::new(catalog.triggers));
    judge_file_with_prepared_triggers(file, catalog, installed.as_ref())
}

pub(crate) fn judge_file_with_prepared_triggers(
    file: &SqlStatementFileFacts,
    catalog: &Catalog<'_>,
    installed: Option<&InstalledTriggers>,
) -> Vec<(SqlFactSite, usize, String)> {
    let mut findings = Vec::new();
    if file.parse_failed && !file.parse_failures.is_empty() {
        findings.extend(
            file.parse_failures
                .iter()
                .enumerate()
                .filter(|(_, failure)| {
                    failure.insert_keyword_count > failure.recovered_insert_count
                        && (failure.insert_keyword_count > 1 || !failure.has_top_level_not_exists)
                })
                .map(|(index, failure)| {
                    (
                        SqlFactSite::ParseFailure(index),
                        failure.line.max(1),
                        if failure.insert_keyword_count > 1 {
                            "unparseable fragment carries more than one INSERT; hoist each statement"
                        } else {
                            "INSERT could not be proven replay-safe"
                        }.to_string(),
                    )
                }),
        );
    } else if file.parse_failed && file.insert_keyword_count > file.inserts.len() {
        if file.insert_keyword_count > 1 {
            findings.push((
                SqlFactSite::Origin,
                file.origin_line.max(1),
                "unparseable fragment carries more than one INSERT; hoist each statement"
                    .to_string(),
            ));
            return findings;
        }
        if file.has_top_level_not_exists {
            return findings;
        }
        findings.push((
            SqlFactSite::Origin,
            file.origin_line.max(1),
            "INSERT could not be proven replay-safe".to_string(),
        ));
        return findings;
    }
    for (index, insert) in file.inserts.iter().enumerate() {
        if !insert.executed {
            continue;
        }
        if let Some(message) = judge_insert(insert, catalog, installed, &file.path) {
            findings.push((SqlFactSite::Insert(index), insert.line.max(1), message));
        }
    }
    findings
}

fn judge_insert(
    insert: &SqlInsertFact,
    catalog: &Catalog<'_>,
    installed: Option<&InstalledTriggers<'_>>,
    path: &std::path::Path,
) -> Option<String> {
    let triggers = || {
        installed
            .into_iter()
            .flat_map(|prepared| prepared.at(path, insert.line, &insert.table))
    };
    if insert.guarded_select {
        return if catalog.check_triggers {
            trigger::judge_guarded_select_with_triggers(insert, catalog, triggers())
        } else {
            None
        };
    }
    let Some(conflict) = &insert.on_conflict else {
        return Some("INSERT must include ON CONFLICT or a conjunctive WHERE NOT EXISTS".into());
    };
    if conflict.action == SqlOnConflictAction::DoNothing {
        return if catalog.check_triggers {
            trigger::judge_with_triggers(insert, conflict, catalog, triggers())
        } else {
            None
        };
    }
    if catalog.check_arbiter {
        if let Some(message) = arbiter::judge(insert, conflict) {
            return Some(message);
        }
    }
    if catalog.check_convergence || catalog.check_volatility {
        if let Some(message) = convergence::judge(
            conflict,
            catalog.check_convergence,
            catalog.check_volatility,
        ) {
            return Some(message);
        }
    }
    if catalog.check_generated {
        if let Some(message) = generated::judge(insert, conflict, catalog.schema) {
            return Some(message);
        }
    }
    if catalog.check_triggers {
        if let Some(message) = trigger::judge_with_triggers(insert, conflict, catalog, triggers()) {
            return Some(message);
        }
    }
    None
}

pub fn form_is_excluded(form: &SqlValueForm, column: &str) -> bool {
    matches!(form, SqlValueForm::Excluded { column: name } if name.eq_ignore_ascii_case(column))
}

pub fn form_is_self(form: &SqlValueForm, column: &str) -> bool {
    matches!(form, SqlValueForm::SelfRef { column: name } if name.eq_ignore_ascii_case(column))
}

#[cfg(test)]
mod tests;
