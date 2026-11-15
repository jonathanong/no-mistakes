//! An allow entry the domain proof makes redundant.
use super::evaluate::DomainAccept;
use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::CatalogObjectRef;
use crate::codebase::rules::config_diagnostic::ConfigAnchor;
use crate::codebase::rules::PostgresFindings;
use std::collections::BTreeSet;

pub(super) struct Plan {
    pub(super) consumed: Vec<String>,
    pub(super) findings: PostgresFindings,
}

pub(super) fn note(accepted: &mut BTreeSet<String>, accepts: &[DomainAccept]) {
    for accept in accepts {
        accepted.insert(CatalogObjectRef::Table(accept.table.clone()).to_string());
    }
}

pub(super) fn plan(
    findings: &PostgresFindings,
    advice: &PostgresFindings,
    accepted: &BTreeSet<String>,
    opts: &CompiledOptions,
    anchor: &ConfigAnchor,
) -> Plan {
    let mut residual = BTreeSet::new();
    for located in findings.0.iter().chain(advice.0.iter()) {
        if let Some(target) = located.finding.target.as_deref() {
            residual.insert(target.to_string());
        }
    }
    let mut consumed = Vec::new();
    let mut extra = PostgresFindings::default();
    for (index, object) in opts.allow.entry_objects() {
        if !accepted.contains(object) || residual.contains(object) {
            continue;
        }
        let line = anchor.option_item_line("allow", index);
        extra.push(anchor.emit(
            RULE_ID,
            line,
            format!("stale {RULE_ID} allow entry: {object}"),
            None,
            Some(object.to_string()),
        ));
        consumed.push(object.to_string());
    }
    Plan {
        consumed,
        findings: extra,
    }
}
