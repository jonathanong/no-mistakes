use super::super::super::declared::{self, Gate};
use super::super::super::RULE_ID;
use super::diagnostic;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::rules::PostgresFindings;
use crate::codebase::ts_source::matching_disable_directive;

pub(super) fn apply(
    prepared: &dyn SqlFactContext,
    findings: &mut PostgresFindings,
    source: Option<&str>,
    rel: &str,
    anchor: usize,
    gate: &Gate,
    offenders_present: bool,
) {
    match gate {
        Gate::Valid {
            line,
            rows,
            justification,
        } => {
            if covered(source, *line) || covered(source, anchor) {
                return;
            }
            if offenders_present {
                prepared.record_accepted_statement_bound(RULE_ID, rel, *line, *rows, justification);
            } else {
                findings.push(diagnostic(rel, *line, &declared::stale_message(*rows)));
            }
        }
        Gate::Invalid { line, message } => findings.push(diagnostic(rel, *line, message)),
        Gate::Blocked { findings: blocked } => {
            for (line, message) in blocked {
                findings.push(diagnostic(rel, *line, message));
            }
        }
    }
}

pub(super) fn clears(source: Option<&str>, anchor: usize, gate: &Gate) -> bool {
    let Gate::Valid { line, .. } = gate else {
        return false;
    };
    !covered(source, *line) && !covered(source, anchor)
}

fn covered(source: Option<&str>, line: usize) -> bool {
    let Some(source) = source else {
        return false;
    };
    let Ok(line) = u32::try_from(line) else {
        return false;
    };
    matching_disable_directive(source, Some(line), RULE_ID).is_some()
}
