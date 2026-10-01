use crate::codebase::rules::RuleFinding;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogObjectRef {
    Table(String),
    Column { table: String, column: String },
    Index { table: String, index: String },
    Trigger { table: String, trigger: String },
    Constraint { table: String, name: String },
    Function(String),
    Enum(String),
    View(String),
    MaterializedView(String),
}

impl fmt::Display for CatalogObjectRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table(name) => write!(formatter, "table:{name}"),
            Self::Column { table, column } => write!(formatter, "column:{table}.{column}"),
            Self::Index { table, index } => write!(formatter, "index:{table}.{index}"),
            Self::Trigger { table, trigger } => write!(formatter, "trigger:{table}.{trigger}"),
            Self::Constraint { table, name } => write!(formatter, "constraint:{table}.{name}"),
            Self::Function(key) => write!(formatter, "function:{key}"),
            Self::Enum(name) => write!(formatter, "enum:{name}"),
            Self::View(name) => write!(formatter, "view:{name}"),
            Self::MaterializedView(name) => write!(formatter, "materialized-view:{name}"),
        }
    }
}

impl FromStr for CatalogObjectRef {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, ()> {
        if let Some(rest) = raw.strip_prefix("materialized-view:") {
            return Ok(Self::MaterializedView(non_empty(rest)?));
        }
        let (prefix, rest) = raw.split_once(':').ok_or(())?;
        let object = match prefix {
            "table" => Self::Table(non_empty(rest)?),
            "column" => {
                let (table, column) = pair(rest)?;
                Self::Column { table, column }
            }
            "index" => {
                let (table, index) = pair(rest)?;
                Self::Index { table, index }
            }
            "trigger" => {
                let (table, trigger) = pair(rest)?;
                Self::Trigger { table, trigger }
            }
            "constraint" => {
                let (table, name) = pair(rest)?;
                Self::Constraint { table, name }
            }
            "function" => Self::Function(non_empty(rest)?),
            "enum" => Self::Enum(non_empty(rest)?),
            "view" => Self::View(non_empty(rest)?),
            _ => return Err(()),
        };
        Ok(object)
    }
}

fn non_empty(raw: &str) -> Result<String, ()> {
    (!raw.trim().is_empty()).then(|| raw.to_string()).ok_or(())
}

fn pair(raw: &str) -> Result<(String, String), ()> {
    let (left, right) = raw.split_once('.').ok_or(())?;
    Ok((non_empty(left)?, non_empty(right)?))
}

pub fn catalog_finding(
    rule_id: &str,
    catalog_path: &str,
    object: &CatalogObjectRef,
    text: &str,
) -> RuleFinding {
    let path = slash_normalize(catalog_path);
    let object = object.to_string();
    RuleFinding {
        rule: rule_id.to_string(),
        file: path.clone(),
        line: 1,
        message: format!("{path}: {object}: {text}"),
        import: None,
        target: Some(object),
    }
}

pub fn require_catalog_path(rule_id: &str, path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("{rule_id} option schemaCatalogPath: required");
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowEntry {
    pub object: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct AllowList {
    rule_id: String,
    entries: Vec<AllowEntry>,
}

impl AllowList {
    pub fn compile(rule_id: &str, entries: Vec<AllowEntry>) -> Result<Self> {
        let mut seen = BTreeSet::new();
        for entry in &entries {
            if entry.reason.trim().is_empty() {
                bail!(
                    "{rule_id} option allow: entry {} needs a reason",
                    entry.object
                );
            }
            if entry.object.parse::<CatalogObjectRef>().is_err() {
                bail!(
                    "{rule_id} option allow: invalid object ref {}",
                    entry.object
                );
            }
            if !seen.insert(entry.object.clone()) {
                bail!("{rule_id} option allow: duplicate entry {}", entry.object);
            }
        }
        Ok(Self {
            rule_id: rule_id.to_string(),
            entries,
        })
    }

    pub fn apply(self, catalog_path: &str, mut findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
        let path = slash_normalize(catalog_path);
        let mut used = vec![false; self.entries.len()];
        findings.retain(|finding| {
            let Some(target) = finding.target.as_deref() else {
                return true;
            };
            let mut matched = false;
            for (index, entry) in self.entries.iter().enumerate() {
                if entry.object == target {
                    used[index] = true;
                    matched = true;
                }
            }
            !matched
        });
        for (entry, was_used) in self.entries.iter().zip(used) {
            if was_used {
                continue;
            }
            findings.push(RuleFinding {
                rule: self.rule_id.clone(),
                file: path.clone(),
                line: 1,
                message: format!(
                    "{path}: stale {} allow entry: {}",
                    self.rule_id, entry.object
                ),
                import: None,
                target: Some(entry.object.clone()),
            });
        }
        findings
    }
}

fn slash_normalize(path: &str) -> String {
    path.replace('\\', "/")
}
