use crate::codebase::postgres::embedded::{
    placeholders, EmbeddedSqlSourcePosition, EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS,
};
mod append_effects;
mod conditional;
mod conditions;
pub(super) mod control;
mod effects;
mod expression;
mod helper;
mod helper_alias;
mod helper_bindings;
mod helper_effects;
mod limit;
mod logical;
use limit::bounded;
mod origins;
mod recovery;
mod state;
pub(super) mod switch;
mod template;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ValueKind {
    Sql,
    Null,
    Boolean(bool),
    Absent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Recovered {
    pub value: ValueKind,
    pub sql: String,
    pub line: u32,
    pub origins: Vec<u32>,
    pub positions: Vec<EmbeddedSqlSourcePosition>,
    pub fragment: bool,
    pub enumerated: bool,
    pub choices: Vec<(u64, u32)>,
    pub append_sites: Vec<u32>,
}

impl Recovered {
    fn empty(line: u32) -> Self {
        Self {
            value: ValueKind::Sql,
            sql: String::new(),
            line,
            origins: Vec::new(),
            positions: Vec::new(),
            fragment: false,
            enumerated: false,
            choices: Vec::new(),
            append_sites: Vec::new(),
        }
    }
    pub(super) fn truth(&self) -> bool {
        match self.value {
            ValueKind::Sql => self.fragment || !self.sql.is_empty(),
            ValueKind::Boolean(value) => value,
            ValueKind::Null | ValueKind::Absent => false,
        }
    }
    fn append_sql(&mut self, suffix: &Self) {
        if self.sql.is_empty() && self.positions.is_empty() {
            self.line = suffix.line;
        }
        let offset = placeholders::count_placeholders(&self.sql);
        self.origins.extend(origins::rewrite(
            &suffix.sql,
            &suffix.origins,
            offset,
            false,
        ));
        let mut positions = suffix.positions.clone();
        super::resolve::append::positions::renumber(&mut positions, &suffix.sql, offset);
        super::resolve::append::positions::append(
            &mut self.positions,
            &self.sql,
            self.line,
            suffix.line,
            &positions,
        );
        self.sql
            .push_str(&placeholders::renumber_placeholders(&suffix.sql, offset));
        self.merge_append_sites(suffix);
        self.fragment |= suffix.fragment;
        self.enumerated |= suffix.enumerated;
        for choice in &suffix.choices {
            if !self.choices.contains(choice) {
                self.choices.push(*choice);
            }
        }
    }
    pub(super) fn merge_append_sites(&mut self, other: &Self) {
        for site in &other.append_sites {
            if !self.append_sites.contains(site) {
                self.append_sites.push(*site);
            }
        }
    }
    pub(super) fn same_value(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.append_sites.clear();
        right.append_sites.clear();
        left == right
    }
    pub(super) fn publish(self) -> EmbeddedSqlVariant {
        let sql_source_offsets = origins::rewrite(&self.sql, &self.origins, 0, true);
        let (sql_text, recovered_placeholder_positions) =
            placeholders::publish_placeholders_with_positions(self.sql);
        EmbeddedSqlVariant {
            sql_text,
            sql_source_offsets,
            line: self.line,
            sql_source_positions: self.positions,
            recovered_placeholder_positions,
            append_sites: self.append_sites,
        }
    }
}

fn alternatives(left: Vec<Recovered>, right: Vec<Recovered>) -> Option<Vec<Recovered>> {
    bounded(left.into_iter().chain(right).collect())
}

fn combine(left: Vec<Recovered>, right: Vec<Recovered>) -> Option<Vec<Recovered>> {
    if left
        .iter()
        .chain(&right)
        .any(|value| value.value != ValueKind::Sql)
    {
        return None;
    }
    let mut out = Vec::new();
    for prefix in left {
        for suffix in &right {
            if prefix.choices.iter().any(|(id, arm)| {
                suffix
                    .choices
                    .iter()
                    .any(|(other, branch)| id == other && arm != branch)
            }) {
                continue;
            }
            let mut value = prefix.clone();
            value.append_sql(suffix);
            out.push(value);
            if out.len() > MAX_EMBEDDED_SQL_VARIANTS {
                return None;
            }
        }
    }
    bounded(out)
}
