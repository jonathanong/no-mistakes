use super::super::RULE_ID;
use crate::codebase::postgres::{TriggerEvent, TriggerTiming};
use anyhow::{bail, Result};
use regex::Regex;

pub(super) fn present_type(data_type: Option<&str>) -> Result<Option<String>> {
    let Some(data_type) = data_type else {
        return Ok(None);
    };
    if data_type.trim().is_empty() {
        bail!("{RULE_ID} option type: required");
    }
    Ok(Some(data_type.to_string()))
}

pub(super) fn foreign_key_mode(configured: Option<bool>, implied: bool) -> Result<Option<bool>> {
    if configured == Some(false) && implied {
        bail!("{RULE_ID} option foreignKey: false conflicts with references or onDelete");
    }
    Ok(match configured {
        Some(value) => Some(value),
        None if implied => Some(true),
        None => None,
    })
}

pub(super) fn parse_on_delete(raw: &str) -> Result<String> {
    let value = raw.trim().to_ascii_lowercase();
    match value.as_str() {
        "cascade" | "restrict" | "set null" | "set default" | "no action" => Ok(value),
        _ => bail!("{RULE_ID} option onDelete: unknown value {raw}"),
    }
}

pub(super) fn parse_timing(raw: &str) -> Result<TriggerTiming> {
    match raw {
        "before" => Ok(TriggerTiming::Before),
        "after" => Ok(TriggerTiming::After),
        "instead-of" => Ok(TriggerTiming::InsteadOf),
        _ => bail!("{RULE_ID} option timing: unknown value {raw}"),
    }
}

pub(super) fn parse_events(events: Option<&[String]>) -> Result<Vec<TriggerEvent>> {
    let Some(events) = events else {
        return Ok(vec![TriggerEvent::Update]);
    };
    if events.is_empty() {
        bail!("{RULE_ID} option events: must not be empty");
    }
    events
        .iter()
        .map(|event| match event.as_str() {
            "insert" => Ok(TriggerEvent::Insert),
            "update" => Ok(TriggerEvent::Update),
            "delete" => Ok(TriggerEvent::Delete),
            "truncate" => Ok(TriggerEvent::Truncate),
            _ => bail!("{RULE_ID} option events: unknown event {event}"),
        })
        .collect()
}

pub(super) fn regex_of(option: &str, pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|error| {
        anyhow::anyhow!("{RULE_ID} option {option}: invalid regex {pattern}: {error}")
    })
}
