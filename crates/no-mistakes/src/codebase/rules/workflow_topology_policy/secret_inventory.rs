use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

mod evaluate;
mod findings;
mod resolve;
pub(super) use evaluate::lint;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SecretInventory {
    repository: Vec<String>,
    environments: BTreeMap<String, Vec<String>>,
    built_in_exemptions: Option<Vec<String>>,
    report_unused: bool,
}

pub(super) struct CompiledInventory {
    repository: BTreeMap<String, String>,
    environments: BTreeMap<String, (String, BTreeMap<String, String>)>,
    exemptions: BTreeSet<String>,
    report_unused: bool,
}

impl SecretInventory {
    pub(super) fn compile(&self) -> Result<CompiledInventory> {
        let repository = names(&self.repository, "repository")?;
        let mut environments = BTreeMap::new();
        for (environment, secrets) in &self.environments {
            if environment.trim().is_empty() {
                bail!("secretInventory environment names must not be empty");
            }
            let key = environment.to_ascii_lowercase();
            if environments
                .insert(key, (environment.clone(), names(secrets, environment)?))
                .is_some()
            {
                bail!("duplicate secretInventory environment: {environment}");
            }
        }
        let defaults = vec!["GITHUB_TOKEN".to_owned()];
        let exemptions = names(
            self.built_in_exemptions.as_ref().unwrap_or(&defaults),
            "builtInExemptions",
        )?
        .into_keys()
        .collect();
        Ok(CompiledInventory {
            repository,
            environments,
            exemptions,
            report_unused: self.report_unused,
        })
    }
}

fn names(values: &[String], scope: &str) -> Result<BTreeMap<String, String>> {
    let mut names = BTreeMap::new();
    for name in values {
        let mut bytes = name.bytes();
        if !bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
            || !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            bail!("invalid secretInventory name in {scope}: {name:?}");
        }
        if names
            .insert(name.to_ascii_lowercase(), name.clone())
            .is_some()
        {
            bail!("duplicate secretInventory name in {scope}: {name}");
        }
    }
    Ok(names)
}
