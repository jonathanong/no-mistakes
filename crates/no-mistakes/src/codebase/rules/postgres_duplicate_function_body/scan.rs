use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct GroupKey {
    language: String,
    search_path: String,
    security: String,
    parallel: String,
    null_input: String,
    volatility: String,
    leakproof: String,
    return_contract: String,
    planner: String,
    kind: &'static str,
    form: &'static str,
    tokens: Vec<String>,
}

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut groups: BTreeMap<GroupKey, Vec<String>> = BTreeMap::new();
    for function in catalog.functions() {
        let Some(body) = function.body.as_deref() else {
            continue;
        };
        let language = function.language.clone().unwrap_or_default();
        if !language.is_empty() && !matches!(language.as_str(), "sql" | "plpgsql") {
            continue;
        }
        let Some(tokens) = super::normalize::normalized_tokens(
            body,
            &compiled.settings,
            function.language.as_deref(),
        ) else {
            continue;
        };
        if tokens.len() < compiled.min_tokens {
            continue;
        }
        let kind = function_kind(function.returns_trigger, function.returns_event_trigger);
        let outside = outside_body(&function.definition, function.body_span);
        groups
            .entry(GroupKey {
                language,
                search_path: search_path(&outside),
                security: function.security.clone(),
                parallel: function.parallel.clone(),
                null_input: function.null_input.clone(),
                volatility: function.volatility.clone(),
                leakproof: function.leakproof.clone(),
                return_contract: function.return_contract.clone(),
                planner: function.planner.clone(),
                kind,
                form: body_form(&function.definition, function.body_span),
                tokens,
            })
            .or_default()
            .push(function.key.clone());
    }
    let mut findings = Vec::new();
    for (key, members) in groups {
        if members.len() < compiled.min_cluster_size {
            continue;
        }
        let count = members.len() - 1;
        for (index, name) in members.iter().enumerate() {
            let names = other_names(&members, index);
            let text = compiled
                .message
                .clone()
                .unwrap_or_else(|| finding_text(count, &names, key.kind));
            findings.push(catalog_finding(
                RULE_ID,
                &compiled.schema_catalog_path,
                &CatalogObjectRef::Function(name.clone()),
                &text,
            ));
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn other_names(members: &[String], index: usize) -> Vec<&str> {
    let mut names = Vec::new();
    for (other_index, other) in members.iter().enumerate() {
        if other_index == index {
            continue;
        }
        names.push(other.as_str());
        if names.len() == 5 {
            break;
        }
    }
    names
}

fn body_form(definition: &str, span: Option<(usize, usize)>) -> &'static str {
    let Some((start, _)) = span else {
        return "string";
    };
    let tail = definition.get(..start).unwrap_or("").trim_end();
    if ends_with_word(tail, "atomic") || ends_with_word(tail, "return") {
        "parsed"
    } else {
        "string"
    }
}

fn ends_with_word(text: &str, word: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let Some(rest) = lower.strip_suffix(word) else {
        return false;
    };
    rest.is_empty() || rest.ends_with(char::is_whitespace)
}

pub(super) fn outside_body(definition: &str, span: Option<(usize, usize)>) -> String {
    let Some((start, end)) = span else {
        return definition.to_string();
    };
    if start > end || end > definition.len() {
        return definition.to_string();
    }
    let mut text = String::with_capacity(definition.len());
    text.push_str(&definition[..start]);
    text.push(' ');
    text.push_str(&definition[end..]);
    text
}

fn function_kind(returns_trigger: bool, returns_event_trigger: bool) -> &'static str {
    if returns_trigger {
        "trigger"
    } else if returns_event_trigger {
        "event"
    } else {
        "routine"
    }
}

fn search_path(header: &str) -> String {
    super::search_path::extract(header)
}

fn finding_text(count: usize, others: &[&str], kind: &str) -> String {
    let listed = if count <= 5 {
        others.join(", ")
    } else {
        format!("{} and {} more", others.join(", "), count - 5)
    };
    let remedy = match kind {
        "trigger" => "replace them with one function parameterized by TG_TABLE_NAME / TG_ARGV",
        "event" => {
            "replace them with one event trigger function; event triggers cannot take arguments"
        }
        _ => "replace them with one function that takes the varying values as arguments",
    };
    format!(
        "function body duplicates {count} other function(s) after normalizing names and literals: {listed}. Copied functions drift, so a fix has to be repeated in each copy; {remedy}",
    )
}
