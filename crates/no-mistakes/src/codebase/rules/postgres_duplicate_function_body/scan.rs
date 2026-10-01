use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

type Member = (String, bool);

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut groups: BTreeMap<(String, Vec<String>), Vec<Member>> = BTreeMap::new();
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
            .entry((language, tokens))
            .or_default()
            .push((function.key.clone(), function.returns_trigger));
    }
    let mut findings = Vec::new();
    for members in groups.into_values() {
        if members.len() < compiled.min_cluster_size {
            continue;
        }
        for (key, returns_trigger) in &members {
            let others = members
                .iter()
                .filter(|(other, _)| other != key)
                .map(|(other, _)| other.as_str())
                .collect::<Vec<_>>();
            let text = compiled
                .message
                .clone()
                .unwrap_or_else(|| finding_text(&others, *returns_trigger));
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

fn finding_text(others: &[&str], returns_trigger: bool) -> String {
    let listed = if others.len() <= 5 {
        others.join(", ")
    } else {
        format!("{} and {} more", others[..5].join(", "), others.len() - 5)
    };
    let remedy = if returns_trigger {
        "replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV"
    } else {
        "replace them with one function that takes the varying values as arguments"
    };
    format!(
        "function body duplicates {} other function(s) after normalising names and literals: {listed}; {remedy}",
        others.len()
    )
}
