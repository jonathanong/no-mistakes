use super::compile::Compiled;
use super::text::{literal_text, name_text};
use super::RULE_ID;
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

pub(super) struct Seen {
    pub(super) table: String,
    pub(super) column: String,
    pub(super) data_type: String,
    pub(super) candidate: bool,
    pub(super) ignored: bool,
    pub(super) values: Vec<String>,
}

pub(super) fn scan(
    catalog: &SchemaCatalog,
    compiled: &Compiled,
    path: &str,
    usage: &super::super::allow_audit::AllowUsage,
) -> Vec<RuleFinding> {
    let seen = super::collect::collect(catalog, compiled);
    let peers = peer_index(&seen);
    let mut findings = Vec::new();
    for item in &seen {
        if !item.candidate {
            continue;
        }
        let violates = !item.values.is_empty() || compiled.matching_pattern(&item.column).is_some();
        if violates
            && usage.suppress(
                compiled
                    .ignore
                    .iter()
                    .map(|pattern| pattern.is_match(&item.table)),
            )
        {
            continue;
        }
        let object = CatalogObjectRef::Column {
            table: item.table.clone(),
            column: item.column.clone(),
        };
        if item.values.is_empty() {
            if let Some(pattern) = compiled.matching_pattern(&item.column) {
                findings.push(column_finding(
                    catalog,
                    RULE_ID,
                    path,
                    &object,
                    &item.table,
                    &item.column,
                    &name_text(&item.data_type, pattern),
                ));
            }
            continue;
        }
        findings.push(column_finding(
            catalog,
            RULE_ID,
            path,
            &object,
            &item.table,
            &item.column,
            &literal_text(&item.data_type, &item.values, &peers_for(&peers, item)),
        ));
    }
    super::super::sort_findings(&mut findings);
    if let Some(message) = compiled
        .message
        .as_deref()
        .filter(|message| !message.trim().is_empty())
    {
        for finding in &mut findings {
            finding.message = format!(
                "{}: {}: {message}",
                finding.file,
                finding.target.as_deref().unwrap_or_default()
            );
        }
    }
    let mut findings = compiled.allow.clone().apply(path, findings);
    super::super::sort_findings(&mut findings);
    findings
}

fn column_finding(
    catalog: &SchemaCatalog,
    rule: &str,
    path: &str,
    object: &CatalogObjectRef,
    table: &str,
    column: &str,
    text: &str,
) -> RuleFinding {
    let mut finding = catalog_finding(rule, path, object, text);
    finding.line = catalog.column_line(table, column);
    finding
}

type PeerIndex = BTreeMap<Vec<String>, Vec<(String, String)>>;

fn peer_index(seen: &[Seen]) -> PeerIndex {
    let mut index: PeerIndex = BTreeMap::new();
    for item in seen
        .iter()
        .filter(|item| item.candidate && !item.ignored && !item.values.is_empty())
    {
        let mut key = item.values.clone();
        key.sort();
        key.dedup();
        index.entry(key).or_default().push((
            format!("column:{}.{}", item.table, item.column),
            format!("{}.{}", item.table, item.column),
        ));
    }
    for peers in index.values_mut() {
        peers.sort();
    }
    index
}

fn peers_for(index: &PeerIndex, item: &Seen) -> Vec<String> {
    let mut key = item.values.clone();
    key.sort();
    key.dedup();
    index
        .get(&key)
        .into_iter()
        .flatten()
        .filter(|(sort_key, _)| sort_key != &format!("column:{}.{}", item.table, item.column))
        .map(|(_, display)| display.clone())
        .collect()
}
