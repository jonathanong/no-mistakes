use super::{Options, CHECKS, RULE_ID};
use serde::Deserialize;
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Mode {
    Fail,
    #[default]
    Advise,
    Ignore,
}
#[derive(Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Allow {
    pub relation: Option<String>,
    pub key: Option<String>,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub reason: String,
}
#[derive(Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WriteSet {
    pub file: String,
    pub transaction_line: u32,
    pub relations: Vec<String>,
    pub reason: Option<String>,
}
pub(super) fn validate(options: &Options) -> anyhow::Result<()> {
    for check in options.checks.keys() {
        anyhow::ensure!(
            CHECKS.contains(&check.as_str()),
            "{RULE_ID}: unknown check {check}"
        );
    }
    for entry in &options.allow {
        anyhow::ensure!(
            !entry.reason.trim().is_empty(),
            "{RULE_ID}: allow entries require a reason"
        );
        anyhow::ensure!(
            entry.relation.is_some()
                || entry.key.is_some()
                || entry.file.is_some()
                || entry.line.is_some(),
            "{RULE_ID}: allow entries require a relation, key, file or line"
        );
        if let Some(relation) = &entry.relation {
            valid_relation(relation)?;
        }
    }
    for entry in &options.write_sets {
        anyhow::ensure!(
            entry
                .reason
                .as_deref()
                .is_none_or(|reason| !reason.trim().is_empty()),
            "{RULE_ID}: writeSets reason must be nonempty when supplied"
        );
        anyhow::ensure!(
            !entry.file.is_empty() && entry.transaction_line > 0 && !entry.relations.is_empty(),
            "{RULE_ID}: writeSets require file, positive transactionLine and relations"
        );
        for relation in &entry.relations {
            valid_relation(relation)?;
        }
    }
    Ok(())
}
fn valid_relation(relation: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        crate::codebase::postgres::idents::parse_relation_identity(relation).is_some(),
        "{RULE_ID}: invalid relation {relation}"
    );
    Ok(())
}
pub(super) fn relation_matches(configured: &str, relation: &str) -> bool {
    crate::codebase::postgres::idents::parse_relation_identity(configured).as_deref()
        == Some(relation)
}
impl Allow {
    pub fn matches(
        &self,
        file: &str,
        line: usize,
        relation: Option<&str>,
        key: Option<&str>,
    ) -> bool {
        self.file.as_deref().is_none_or(|value| value == file)
            && self.line.is_none_or(|value| value == line)
            && self.relation.as_deref().is_none_or(|value| {
                relation.is_some_and(|relation| relation_matches(value, relation))
            })
            && self.key.as_deref().is_none_or(|value| Some(value) == key)
    }
}
