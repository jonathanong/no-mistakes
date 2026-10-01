mod parse;

use super::{BannedOptions, ColumnOptions, Options, ShapeOptions, TriggerOptions, RULE_ID};
use crate::codebase::postgres::{AllowList, TriggerEvent, TriggerTiming};
use anyhow::{bail, Result};
use parse::{
    foreign_key_mode, parse_events, parse_on_delete, parse_timing, present_type, regex_of,
};
use regex::Regex;

pub(super) struct Shape {
    pub(super) name: String,
    pub(super) table_pattern: Regex,
    pub(super) columns: Vec<RequiredColumn>,
    pub(super) primary_key_types: Vec<String>,
    pub(super) forbidden: Vec<String>,
    pub(super) triggers: Vec<TriggerNeed>,
}

pub(super) struct RequiredColumn {
    pub(super) name: Option<String>,
    pub(super) name_pattern: Option<Regex>,
    pub(super) pattern_source: String,
    pub(super) data_type: Option<String>,
    pub(super) nullable: Option<bool>,
    pub(super) foreign_key: Option<bool>,
    pub(super) on_delete: Option<String>,
    pub(super) references: Option<Vec<String>>,
}

pub(super) struct TriggerNeed {
    pub(super) function: String,
    pub(super) timing: TriggerTiming,
    pub(super) events: Vec<TriggerEvent>,
    pub(super) for_each_row: bool,
}

pub(super) struct Banned {
    pub(super) pattern: Regex,
    pub(super) source: String,
    pub(super) message: String,
}

pub(super) fn compile(opts: &Options, message: Option<String>) -> Result<super::Compiled> {
    crate::codebase::postgres::require_catalog_path(RULE_ID, &opts.schema_catalog_path)?;
    let mut names = Vec::new();
    let mut shapes = Vec::new();
    for shape in &opts.shapes {
        let name = shape.name.trim();
        if name.is_empty() {
            bail!("{RULE_ID} option name: required");
        }
        if names.iter().any(|existing: &String| existing == name) {
            bail!("{RULE_ID} option name: duplicate entry {name}");
        }
        names.push(name.to_string());
        shapes.push(compile_shape(shape, name)?);
    }
    let banned = opts
        .banned_table_patterns
        .iter()
        .map(compile_banned)
        .collect::<Result<Vec<_>>>()?;
    Ok(super::Compiled {
        schema_catalog_path: opts.schema_catalog_path.clone(),
        shapes,
        banned,
        allow: AllowList::compile(RULE_ID, opts.allow.clone())?,
        message,
    })
}

fn compile_shape(shape: &ShapeOptions, name: &str) -> Result<Shape> {
    if shape.table_pattern.trim().is_empty() {
        bail!("{RULE_ID} option tablePattern: required");
    }
    for kind in &shape.primary_key_types {
        if kind.trim().is_empty() {
            bail!("{RULE_ID} option primaryKeyTypes: empty string");
        }
    }
    Ok(Shape {
        name: name.to_string(),
        table_pattern: regex_of("tablePattern", &shape.table_pattern)?,
        columns: shape
            .required_columns
            .iter()
            .map(compile_column)
            .collect::<Result<Vec<_>>>()?,
        primary_key_types: shape.primary_key_types.clone(),
        forbidden: shape.forbidden_columns.clone(),
        triggers: shape
            .required_triggers
            .iter()
            .map(compile_trigger)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn compile_column(column: &ColumnOptions) -> Result<RequiredColumn> {
    match (column.name.as_ref(), column.name_pattern.as_ref()) {
        (Some(name), None) if !name.trim().is_empty() => {}
        (None, Some(pattern)) if !pattern.trim().is_empty() => {}
        (Some(_), Some(_)) | (None, None) => {
            bail!("{RULE_ID} option requiredColumns: exactly one of name or namePattern");
        }
        _ => bail!("{RULE_ID} option requiredColumns: exactly one of name or namePattern"),
    }
    let references = match &column.references {
        Some(tables) if tables.is_empty() => {
            bail!("{RULE_ID} option references: must not be empty");
        }
        Some(tables) if tables.iter().any(|table| table.trim().is_empty()) => {
            bail!("{RULE_ID} option references: blank table");
        }
        Some(tables) => Some(tables.clone()),
        None => None,
    };
    let on_delete = column
        .on_delete
        .as_deref()
        .map(parse_on_delete)
        .transpose()?;
    Ok(RequiredColumn {
        name: column.name.clone(),
        pattern_source: column.name_pattern.clone().unwrap_or_default(),
        name_pattern: column
            .name_pattern
            .as_deref()
            .map(|pattern| regex_of("namePattern", pattern))
            .transpose()?,
        data_type: present_type(column.data_type.as_deref())?,
        nullable: column.nullable,
        foreign_key: foreign_key_mode(
            column.foreign_key,
            on_delete.is_some() || references.is_some(),
        )?,
        on_delete,
        references,
    })
}

fn compile_trigger(trigger: &TriggerOptions) -> Result<TriggerNeed> {
    if trigger.function.trim().is_empty() {
        bail!("{RULE_ID} option function: required");
    }
    let events = parse_events(trigger.events.as_deref())?;
    let for_each_row = trigger.for_each_row.unwrap_or(true);
    if for_each_row && events.contains(&TriggerEvent::Truncate) {
        bail!("{RULE_ID} option forEachRow: truncate triggers are FOR EACH STATEMENT");
    }
    Ok(TriggerNeed {
        function: super::name::normalize_function_name(&trigger.function),
        timing: parse_timing(trigger.timing.as_deref().unwrap_or("before"))?,
        events,
        for_each_row,
    })
}

fn compile_banned(banned: &BannedOptions) -> Result<Banned> {
    if banned.pattern.trim().is_empty() {
        bail!("{RULE_ID} option pattern: required");
    }
    if banned.message.trim().is_empty() {
        bail!("{RULE_ID} option message: required");
    }
    Ok(Banned {
        pattern: regex_of("pattern", &banned.pattern)?,
        source: banned.pattern.clone(),
        message: banned.message.clone(),
    })
}
