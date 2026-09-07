//! ON CONFLICT replay-safety judgments over statement facts.

mod arbiter;
mod convergence;
mod generated;
mod trigger;

#[cfg(test)]
mod coverage_more_tests;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod trigger_tests;
#[cfg(test)]
mod where_noop_tests;

use crate::codebase::postgres::statement_facts::{
    SqlInsertFact, SqlOnConflictAction, SqlStatementFileFacts, SqlTriggerFact, SqlValueForm,
};
use crate::codebase::postgres::types::SqlSchemaFileFacts;

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
    let mut findings = Vec::new();
    if file.parse_failed && file.insert_keyword_count > file.inserts.len() {
        if file.insert_keyword_count > 1 {
            findings.push((
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
            file.origin_line.max(1),
            "INSERT could not be proven replay-safe".to_string(),
        ));
        return findings;
    }
    for insert in &file.inserts {
        if !insert.executed {
            continue;
        }
        if let Some(message) = judge_insert(insert, catalog) {
            findings.push((insert.line.max(1), message));
        }
    }
    findings
}

fn judge_insert(insert: &SqlInsertFact, catalog: &Catalog<'_>) -> Option<String> {
    if insert.guarded_select {
        return if catalog.check_triggers {
            trigger::judge_guarded_select(insert, catalog)
        } else {
            None
        };
    }
    let Some(conflict) = &insert.on_conflict else {
        return Some("INSERT must include ON CONFLICT or a conjunctive WHERE NOT EXISTS".into());
    };
    if conflict.action == SqlOnConflictAction::DoNothing {
        return if catalog.check_triggers {
            trigger::judge(insert, conflict, catalog)
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
        if let Some(message) = trigger::judge(insert, conflict, catalog) {
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
