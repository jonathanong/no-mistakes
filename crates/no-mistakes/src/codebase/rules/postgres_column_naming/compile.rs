use super::compile_fk::{compile_foreign_keys, ForeignCompiled};
use super::Options;
use crate::codebase::postgres::{require_catalog_path, AllowList};
use anyhow::{bail, Result};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Compiled {
    pub(super) type_rules: Vec<TypeCheck>,
    pub(super) name_type_rules: Vec<NameTypeCheck>,
    pub(super) skip_generated: bool,
    pub(super) ignore: Vec<Regex>,
    pub(super) forbidden: Vec<ForbiddenCheck>,
    pub(super) suffixes: BTreeMap<String, Vec<String>>,
    pub(super) reserved: Vec<ReservedCheck>,
    pub(super) target_mode: TargetMode,
    pub(super) target_names: Vec<TargetNameCheck>,
    pub(super) check_self: bool,
    pub(super) singular: BTreeMap<String, String>,
    pub(super) follow: bool,
    pub(super) require: Option<RequireCheck>,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

pub(super) enum TargetMode {
    Off,
    LastWord,
    FullName,
}

pub(super) struct TypeCheck {
    pub(super) types: Vec<String>,
    pub(super) pattern: Regex,
    pub(super) raw: String,
    pub(super) hint: Option<String>,
}

pub(super) struct NameTypeCheck {
    pub(super) pattern: Regex,
    pub(super) raw: String,
    pub(super) types: Vec<String>,
    pub(super) hint: Option<String>,
}

pub(super) struct ForbiddenCheck {
    pub(super) pattern: Regex,
    pub(super) raw: String,
    pub(super) hint: String,
}

pub(super) struct ReservedCheck {
    pub(super) suffix: String,
    pub(super) types: Vec<String>,
    pub(super) tables: Vec<String>,
    pub(super) hint: Option<String>,
}

pub(super) struct TargetNameCheck {
    pub(super) pattern: Regex,
    pub(super) name: String,
}

pub(super) struct RequireCheck {
    pub(super) types: Vec<String>,
    pub(super) pattern: Regex,
    pub(super) raw: String,
    pub(super) exempt: Vec<ExemptCheck>,
}

pub(super) struct ExemptCheck {
    pub(super) pattern: Regex,
    pub(super) raw: String,
}

pub(super) fn compile(options: &Options, message: Option<String>) -> Result<Compiled> {
    require_catalog_path(super::RULE_ID, &options.schema_catalog_path)?;
    let foreign = compile_foreign_keys(&options.foreign_keys)?;
    assemble(options, foreign, message)
}

fn assemble(
    options: &Options,
    foreign: ForeignCompiled,
    message: Option<String>,
) -> Result<Compiled> {
    Ok(Compiled {
        type_rules: compile_type_rules(&options.type_rules)?,
        name_type_rules: compile_name_types(&options.name_type_rules)?,
        skip_generated: options.skip_generated_columns,
        ignore: compile_ignores(&options.ignore_table_patterns)?,
        forbidden: compile_forbidden(&options.forbidden_column_names)?,
        suffixes: foreign.suffixes,
        reserved: foreign.reserved,
        target_mode: foreign.target_mode,
        target_names: foreign.target_names,
        check_self: foreign.check_self,
        singular: foreign.singular,
        follow: foreign.follow,
        require: foreign.require,
        allow: AllowList::compile(
            super::RULE_ID,
            options
                .allow
                .iter()
                .map(|entry| crate::codebase::postgres::AllowEntry {
                    object: entry.object.clone(),
                    reason: entry.reason.clone(),
                })
                .collect(),
        )?,
        message,
    })
}

fn compile_type_rules(rules: &[super::TypeRule]) -> Result<Vec<TypeCheck>> {
    rules
        .iter()
        .map(|rule| {
            if rule.types.is_empty() {
                bail!("postgres-column-naming option typeRules: empty types");
            }
            let pattern = regex("typeRules", &rule.name_pattern)?;
            Ok(TypeCheck {
                types: rule.types.clone(),
                pattern,
                raw: rule.name_pattern.clone(),
                hint: optional_hint("typeRules", rule.hint.clone())?,
            })
        })
        .collect()
}

fn compile_name_types(rules: &[super::NameTypeRule]) -> Result<Vec<NameTypeCheck>> {
    rules
        .iter()
        .map(|rule| {
            if rule.name_pattern.trim().is_empty() {
                bail!("postgres-column-naming option nameTypeRules: namePattern is required");
            }
            if rule.types.is_empty() {
                bail!("postgres-column-naming option nameTypeRules: empty types");
            }
            let hint = optional_hint("nameTypeRules", rule.hint.clone())?;
            Ok(NameTypeCheck {
                pattern: regex("nameTypeRules", &rule.name_pattern)?,
                raw: rule.name_pattern.clone(),
                types: rule.types.clone(),
                hint,
            })
        })
        .collect()
}

fn compile_forbidden(rules: &[super::ForbiddenName]) -> Result<Vec<ForbiddenCheck>> {
    let mut seen = BTreeSet::new();
    let mut compiled = Vec::new();
    for rule in rules {
        if rule.pattern.trim().is_empty() {
            bail!("postgres-column-naming option forbiddenColumnNames: empty pattern");
        }
        if rule.hint.trim().is_empty() {
            bail!("postgres-column-naming option forbiddenColumnNames: empty hint");
        }
        if !seen.insert(rule.pattern.clone()) {
            bail!(
                "postgres-column-naming option forbiddenColumnNames: duplicate pattern {}",
                rule.pattern
            );
        }
        compiled.push(ForbiddenCheck {
            pattern: regex("forbiddenColumnNames", &rule.pattern)?,
            raw: rule.pattern.clone(),
            hint: rule.hint.clone(),
        });
    }
    Ok(compiled)
}

fn compile_ignores(patterns: &[String]) -> Result<Vec<Regex>> {
    patterns
        .iter()
        .map(|pattern| regex("ignoreTablePatterns", pattern))
        .collect()
}

pub(super) fn regex(option: &str, pattern: &str) -> Result<Regex> {
    if pattern.trim().is_empty() {
        bail!("postgres-column-naming option {option}: empty pattern");
    }
    Regex::new(pattern).map_err(|error| {
        anyhow::anyhow!("postgres-column-naming option {option}: invalid regex: {error}")
    })
}

pub(super) use super::types::lists_type;

pub(super) fn optional_hint(option: &str, hint: Option<String>) -> Result<Option<String>> {
    match hint.as_deref() {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => {
            bail!("postgres-column-naming option {option}: empty hint")
        }
        Some(value) => Ok(Some(value.to_string())),
    }
}
