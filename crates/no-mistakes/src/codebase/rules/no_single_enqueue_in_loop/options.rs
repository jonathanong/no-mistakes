use super::RULE_ID;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Options {
    pub(super) single: Vec<ApiTarget>,
    pub(super) bulk: Vec<ApiTarget>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiTarget {
    pub(super) module: String,
    pub(super) export: String,
}

#[derive(Debug)]
pub(super) struct Targets {
    pub(super) single: Vec<(String, String)>,
    pub(super) bulk: Vec<(String, String)>,
}

pub(super) fn validate(options: &Options) -> Result<Targets> {
    let single = collect("single", &options.single)?;
    let bulk = collect("bulk", &options.bulk)?;
    let single_pairs: BTreeSet<_> = single.iter().cloned().collect();
    if let Some((module, export)) = bulk.iter().find(|pair| single_pairs.contains(pair)) {
        bail!(
            "{RULE_ID}: `{module}` export `{export}` is listed in both options.single and options.bulk"
        );
    }
    Ok(Targets { single, bulk })
}

fn collect(label: &str, targets: &[ApiTarget]) -> Result<Vec<(String, String)>> {
    let mut seen = BTreeSet::new();
    let mut owned = Vec::new();
    for (index, target) in targets.iter().enumerate() {
        let module = target.module.trim();
        let export = target.export.trim();
        if module.is_empty() || export.is_empty() {
            bail!(
                "{RULE_ID}: options.{label}[{index}] needs a non-empty module specifier and export name"
            );
        }
        let pair = (module.to_string(), export.to_string());
        if !seen.insert(pair.clone()) {
            bail!("{RULE_ID}: options.{label} lists `{module}` export `{export}` more than once");
        }
        owned.push(pair);
    }
    Ok(owned)
}
