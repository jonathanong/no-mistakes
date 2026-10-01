use super::concurrency_scope::{self, known_scope};
use super::RULE_ID;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ConcurrencyIntent {
    pub(crate) pending: Option<String>,
    pub(crate) cancellation: Option<String>,
    pub(crate) scope: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompiledIntent {
    pub(super) pending: &'static str,
    pub(super) cancellation: &'static str,
    pub(super) scope: Vec<String>,
}

pub(super) fn compile(
    policy: &BTreeMap<String, ConcurrencyIntent>,
) -> Result<BTreeMap<String, CompiledIntent>> {
    let mut compiled = BTreeMap::new();
    for (id, intent) in policy {
        compiled.insert(id.clone(), compile_intent(id, intent)?);
    }
    Ok(compiled)
}

fn compile_intent(id: &str, intent: &ConcurrencyIntent) -> Result<CompiledIntent> {
    let pending = required(id, "pending", intent.pending.as_deref())?;
    let pending = parse_pending(pending)?;
    let cancellation = required(id, "cancellation", intent.cancellation.as_deref())?;
    let cancellation = parse_cancellation(cancellation)?;
    let scope = intent.scope.as_ref().ok_or_else(|| {
        anyhow::anyhow!("{RULE_ID} option concurrencyPolicy: missing scope: {id}")
    })?;
    Ok(CompiledIntent {
        pending,
        cancellation,
        scope: normalize_scope(id, scope)?,
    })
}

fn required<'a>(id: &str, field: &str, value: Option<&'a str>) -> Result<&'a str> {
    value
        .ok_or_else(|| anyhow::anyhow!("{RULE_ID} option concurrencyPolicy: missing {field}: {id}"))
}

fn parse_pending(value: &str) -> Result<&'static str> {
    match value {
        "coalesce-latest" => Ok("coalesce-latest"),
        "fifo" => Ok("fifo"),
        other => bail!("{RULE_ID} option concurrencyPolicy: unknown pending: {other}"),
    }
}

fn parse_cancellation(value: &str) -> Result<&'static str> {
    match value {
        "cancel-running" => Ok("cancel-running"),
        "retain-running" => Ok("retain-running"),
        "conditional" => Ok("conditional"),
        other => bail!("{RULE_ID} option concurrencyPolicy: unknown cancellation: {other}"),
    }
}

fn normalize_scope(id: &str, entries: &[String]) -> Result<Vec<String>> {
    if entries.is_empty() {
        bail!("{RULE_ID} option concurrencyPolicy: empty scope: {id}");
    }
    let mut seen = BTreeSet::new();
    for entry in entries {
        if !seen.insert(entry.as_str()) {
            bail!("{RULE_ID} option concurrencyPolicy: duplicate scope: {entry}");
        }
        if !known_scope(entry) {
            bail!("{RULE_ID} option concurrencyPolicy: unknown scope: {entry}");
        }
    }
    if entries.iter().any(|entry| entry == "fixed-resource") && entries.len() > 1 {
        bail!(
            "{RULE_ID} option concurrencyPolicy: fixed-resource combined with another scope: {id}"
        );
    }
    if entries.iter().any(|entry| entry == "fixed-resource") {
        return Ok(Vec::new());
    }
    Ok(concurrency_scope::sort_declared(entries))
}
