use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

type Member = (String, bool);

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct GroupKey {
    language: String,
    search_path: String,
    tokens: Vec<String>,
}

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut groups: BTreeMap<GroupKey, Vec<Member>> = BTreeMap::new();
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
        groups
            .entry(GroupKey {
                language,
                search_path: search_path(&function.definition, Some(body)),
                tokens,
            })
            .or_default()
            .push((function.key.clone(), function.returns_trigger));
    }
    let mut findings = Vec::new();
    for members in groups.into_values() {
        if members.len() < compiled.min_cluster_size {
            continue;
        }
        let count = members.len() - 1;
        for (index, (key, returns_trigger)) in members.iter().enumerate() {
            let names = other_names(&members, index);
            let text = compiled
                .message
                .clone()
                .unwrap_or_else(|| finding_text(count, &names, *returns_trigger));
            findings.push(catalog_finding(
                RULE_ID,
                &compiled.schema_catalog_path,
                &CatalogObjectRef::Function(key.clone()),
                &text,
            ));
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn other_names(members: &[Member], index: usize) -> Vec<&str> {
    let mut names = Vec::new();
    for (other_index, (other, _)) in members.iter().enumerate() {
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

fn search_path(definition: &str, body: Option<&str>) -> String {
    let header = body
        .and_then(|body| definition.find(body).map(|index| &definition[..index]))
        .unwrap_or(definition);
    let lower = header.to_ascii_lowercase();
    let Some(start) = lower.find("set search_path") else {
        return String::new();
    };
    if start > 0 && header.as_bytes()[start - 1].is_ascii_alphanumeric() {
        return String::new();
    }
    let after = header[start + "set search_path".len()..].trim_start();
    let lower_after = after.to_ascii_lowercase();
    let mut end = after.len();
    for marker in [" language ", " as ", " as$", " begin ", ";"] {
        if let Some(index) = lower_after.find(marker) {
            end = end.min(index);
        }
    }
    after[..end]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn finding_text(count: usize, others: &[&str], returns_trigger: bool) -> String {
    let listed = if count <= 5 {
        others.join(", ")
    } else {
        format!("{} and {} more", others.join(", "), count - 5)
    };
    let remedy = if returns_trigger {
        "replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV"
    } else {
        "replace them with one function that takes the varying values as arguments"
    };
    format!(
        "function body duplicates {count} other function(s) after normalising names and literals: {listed}; {remedy}",
    )
}
