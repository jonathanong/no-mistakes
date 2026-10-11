use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{
    placeholders, EmbeddedSqlSourcePosition, EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS,
};
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
        self.fragment |= suffix.fragment;
        self.enumerated |= suffix.enumerated;
        for choice in &suffix.choices {
            if !self.choices.contains(choice) {
                self.choices.push(*choice);
            }
        }
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

impl ScopeVisitor<'_> {
    pub(super) fn recover_variants(
        &self,
        expression: &oxc_ast::ast::Expression<'_>,
    ) -> Option<Vec<Recovered>> {
        if self.loop_depth > 0 {
            return None;
        }
        let mut recovered = Vec::new();
        for path in &self.variant_paths {
            let values = expression::recover(self, expression, 8, path)?;
            recovered.extend(values.into_iter().map(|mut value| {
                for choice in path {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                value
            }));
        }
        bounded(recovered)
    }
    pub(super) fn with_variant_paths(&self, values: Vec<Recovered>) -> Option<Vec<Recovered>> {
        let paths: Vec<Recovered> = self
            .variant_paths
            .iter()
            .map(|choices| {
                let mut value = Recovered::empty(1);
                value.choices = choices.clone();
                value
            })
            .collect();
        // Choice-only suffixes do not change the physical mapping.
        let mut result = Vec::new();
        for value in values {
            for path in &paths {
                let path: &Recovered = path;
                if value.choices.iter().any(|(id, arm)| {
                    path.choices
                        .iter()
                        .any(|(other, branch)| id == other && arm != branch)
                }) {
                    continue;
                }
                let mut value = value.clone();
                for choice in &path.choices {
                    if !value.choices.contains(choice) {
                        value.choices.push(*choice);
                    }
                }
                result.push(value);
            }
        }
        bounded(result)
    }
}
