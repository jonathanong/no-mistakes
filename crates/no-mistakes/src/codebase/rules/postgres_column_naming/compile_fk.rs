use super::compile::{
    regex, ExemptCheck, RequireCheck, ReservedCheck, TargetMode, TargetNameCheck,
};
use super::ForeignKeyOptions;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct ForeignCompiled {
    pub(super) suffixes: BTreeMap<String, Vec<String>>,
    pub(super) reserved: Vec<ReservedCheck>,
    pub(super) target_mode: TargetMode,
    pub(super) target_names: Vec<TargetNameCheck>,
    pub(super) check_self: bool,
    pub(super) singular: BTreeMap<String, String>,
    pub(super) follow: bool,
    pub(super) require: Option<RequireCheck>,
}

pub(super) fn compile_foreign_keys(options: &ForeignKeyOptions) -> Result<ForeignCompiled> {
    Ok(ForeignCompiled {
        suffixes: compile_suffixes(options)?,
        reserved: compile_reserved(options)?,
        target_mode: target_mode(&options.target_match)?,
        target_names: compile_target_names(options)?,
        check_self: options.check_self_references,
        singular: options.singular.clone(),
        follow: options.follow_composite_foreign_keys,
        require: compile_require(options)?,
    })
}

fn target_mode(value: &str) -> Result<TargetMode> {
    Ok(match value {
        "off" => TargetMode::Off,
        "last-word" => TargetMode::LastWord,
        "full-name" => TargetMode::FullName,
        other => {
            bail!("postgres-column-naming option foreignKeys.targetMatch: unknown value {other}")
        }
    })
}

fn compile_suffixes(options: &ForeignKeyOptions) -> Result<BTreeMap<String, Vec<String>>> {
    let mut suffixes = BTreeMap::new();
    for entry in &options.target_suffixes {
        if entry.tables.is_empty() {
            bail!("postgres-column-naming option foreignKeys.targetSuffixes: empty tables");
        }
        if entry.suffixes.is_empty() {
            bail!("postgres-column-naming option foreignKeys.targetSuffixes: empty suffixes");
        }
        if entry.suffixes.iter().any(String::is_empty) {
            bail!("postgres-column-naming option foreignKeys.targetSuffixes: empty suffix");
        }
        for table in &entry.tables {
            if suffixes
                .insert(table.clone(), entry.suffixes.clone())
                .is_some()
            {
                bail!(
                    "postgres-column-naming option foreignKeys.targetSuffixes: table {table} listed twice"
                );
            }
        }
    }
    Ok(suffixes)
}

fn compile_reserved(options: &ForeignKeyOptions) -> Result<Vec<ReservedCheck>> {
    let mut seen = BTreeSet::new();
    let mut reserved = Vec::new();
    for entry in &options.reserved_suffixes {
        if entry.suffix.is_empty() {
            bail!("postgres-column-naming option foreignKeys.reservedSuffixes: empty suffix");
        }
        if entry.tables.is_empty() {
            bail!("postgres-column-naming option foreignKeys.reservedSuffixes: empty tables");
        }
        if !seen.insert(entry.suffix.clone()) {
            bail!(
                "postgres-column-naming option foreignKeys.reservedSuffixes: duplicate suffix {}",
                entry.suffix
            );
        }
        reserved.push(ReservedCheck {
            suffix: entry.suffix.clone(),
            types: entry.types.clone(),
            tables: entry.tables.clone(),
            hint: super::compile::optional_hint(
                "foreignKeys.reservedSuffixes",
                entry.hint.clone(),
            )?,
        });
    }
    Ok(reserved)
}

fn compile_target_names(options: &ForeignKeyOptions) -> Result<Vec<TargetNameCheck>> {
    let mut names = Vec::new();
    for entry in &options.target_names {
        if entry.name.is_empty() {
            bail!("postgres-column-naming option foreignKeys.targetNames: empty name");
        }
        let pattern = regex("foreignKeys.targetNames", &entry.table_pattern)?;
        let groups = pattern.captures_len().saturating_sub(1);
        for group in referenced_groups(&entry.name) {
            if group > groups {
                bail!(
                    "postgres-column-naming option foreignKeys.targetNames: ${group} is outside the pattern's groups"
                );
            }
        }
        names.push(TargetNameCheck {
            pattern,
            name: entry.name.clone(),
        });
    }
    Ok(names)
}

fn referenced_groups(name: &str) -> Vec<usize> {
    let chars: Vec<char> = name.chars().collect();
    let mut groups = Vec::new();
    let mut index = 0;
    while index + 1 < chars.len() {
        if chars[index] == '$' && chars[index + 1].is_ascii_digit() {
            let group = chars[index + 1].to_digit(10).expect("ascii digit") as usize;
            if (1..=9).contains(&group) {
                groups.push(group);
            }
        }
        index += 1;
    }
    groups
}

fn compile_require(options: &ForeignKeyOptions) -> Result<Option<RequireCheck>> {
    let Some(require) = &options.require_foreign_key else {
        return Ok(None);
    };
    if require.types.is_empty() {
        bail!("postgres-column-naming option foreignKeys.requireForeignKey: empty types");
    }
    if require.name_pattern.trim().is_empty() {
        bail!(
            "postgres-column-naming option foreignKeys.requireForeignKey: namePattern is required"
        );
    }
    let mut seen = BTreeSet::new();
    let mut exempt = Vec::new();
    for entry in &require.exempt {
        if entry.reason.trim().is_empty() {
            bail!(
                "postgres-column-naming option foreignKeys.requireForeignKey: exempt entry {} needs a reason",
                entry.name_pattern
            );
        }
        if !seen.insert(entry.name_pattern.clone()) {
            bail!(
                "postgres-column-naming option foreignKeys.requireForeignKey: duplicate namePattern {}",
                entry.name_pattern
            );
        }
        exempt.push(ExemptCheck {
            pattern: regex("foreignKeys.requireForeignKey", &entry.name_pattern)?,
            raw: entry.name_pattern.clone(),
        });
    }
    Ok(Some(RequireCheck {
        types: require.types.clone(),
        pattern: regex("foreignKeys.requireForeignKey", &require.name_pattern)?,
        raw: require.name_pattern.clone(),
        exempt,
    }))
}
