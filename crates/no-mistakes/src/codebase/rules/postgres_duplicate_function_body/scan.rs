use super::{Compiled, RULE_ID};
use crate::codebase::postgres::{catalog_finding, CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeMap;

pub(super) fn scan(compiled: Compiled, catalog: &SchemaCatalog) -> Vec<RuleFinding> {
    let mut groups: BTreeMap<(String, Vec<String>), Vec<String>> = BTreeMap::new();
    for function in catalog.functions() {
        let Some(body) = function.body.as_deref() else {
            continue;
        };
        let Some(tokens) = super::normalize::normalized_tokens(body, &compiled.settings) else {
            continue;
        };
        if tokens.len() < compiled.min_tokens {
            continue;
        }
        groups
            .entry((function.language.clone().unwrap_or_default(), tokens))
            .or_default()
            .push(function.key.clone());
    }
    let mut findings = Vec::new();
    for members in groups.into_values() {
        if members.len() < compiled.min_cluster_size {
            continue;
        }
        for key in &members {
            let others = members
                .iter()
                .filter(|other| *other != key)
                .map(String::as_str)
                .collect::<Vec<_>>();
            findings.push(catalog_finding(
                RULE_ID,
                &compiled.schema_catalog_path,
                &CatalogObjectRef::Function(key.clone()),
                &finding_text(&others),
            ));
        }
    }
    compiled
        .allow
        .apply(&compiled.schema_catalog_path, findings)
}

fn finding_text(others: &[&str]) -> String {
    let listed = if others.len() <= 5 {
        others.join(", ")
    } else {
        format!("{} and {} more", others[..5].join(", "), others.len() - 5)
    };
    format!(
        "function body duplicates {} other function(s) after normalising names and literals: {listed}; replace them with one function parameterised by TG_TABLE_NAME / TG_ARGV",
        others.len()
    )
}
