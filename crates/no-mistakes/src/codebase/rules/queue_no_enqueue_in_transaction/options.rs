use super::RULE_ID;
use crate::fx::{FxHashMap, FxHashSet};
use anyhow::{bail, Result};
use serde::Deserialize;

pub(super) type Api<'a> = (&'a str, &'a str);

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Options {
    transactions: Vec<Transaction>,
    begin_transactions: Vec<Publisher>,
    publishers: Vec<Publisher>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Transaction {
    module: String,
    export: String,
    callback_argument: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publisher {
    module: String,
    export: String,
}

pub(super) struct Targets<'a> {
    pub(super) transactions: FxHashMap<Api<'a>, usize>,
    pub(super) begins: FxHashSet<Api<'a>>,
    pub(super) transaction_modules: FxHashSet<&'a str>,
    pub(super) publishers: FxHashSet<Api<'a>>,
    pub(super) publisher_modules: FxHashSet<&'a str>,
}

impl Targets<'_> {
    pub(super) fn has_boundary(&self, api: Api<'_>) -> bool {
        self.transactions.contains_key(&api) || self.begins.contains(&api)
    }
}

impl Options {
    pub(super) fn targets(&self) -> Result<Targets<'_>> {
        if (self.transactions.is_empty() && self.begin_transactions.is_empty())
            || self.publishers.is_empty()
        {
            bail!("{RULE_ID}: options.transactions or options.beginTransactions, and options.publishers, must both name at least one API");
        }
        let mut transactions = FxHashMap::default();
        let mut transaction_modules = FxHashSet::default();
        for transaction in &self.transactions {
            let api = api(&transaction.module, &transaction.export, "transactions")?;
            if transactions
                .insert(api, transaction.callback_argument)
                .is_some()
            {
                bail!(
                    "{RULE_ID}: options.transactions lists `{}#{}` more than once",
                    api.0,
                    api.1
                );
            }
            transaction_modules.insert(api.0);
        }
        let mut begins = FxHashSet::default();
        for begin in &self.begin_transactions {
            let api = api(&begin.module, &begin.export, "beginTransactions")?;
            if transactions.contains_key(&api) || !begins.insert(api) {
                bail!("{RULE_ID}: options.beginTransactions lists `{}#{}` more than once across transaction boundaries", api.0, api.1);
            }
            transaction_modules.insert(api.0);
        }
        let mut publishers = FxHashSet::default();
        let mut publisher_modules = FxHashSet::default();
        for publisher in &self.publishers {
            let api = api(&publisher.module, &publisher.export, "publishers")?;
            publisher_modules.insert(api.0);
            publishers.insert(api);
        }
        Ok(Targets {
            transactions,
            begins,
            transaction_modules,
            publishers,
            publisher_modules,
        })
    }
}

fn api<'a>(module: &'a str, export: &'a str, field: &str) -> Result<Api<'a>> {
    if module.trim().is_empty() || export.trim().is_empty() {
        bail!("{RULE_ID}: options.{field} needs a non-empty module and export");
    }
    Ok((module, export))
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "options_tests.rs"]
mod tests;
