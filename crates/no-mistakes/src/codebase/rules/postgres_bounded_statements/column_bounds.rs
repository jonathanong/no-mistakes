use super::RULE_ID;
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::SchemaCatalog;
use anyhow::{bail, Result};
use serde::Deserialize;
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser, tokenizer::Token};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ColumnBound {
    reason: String,
}

#[derive(Default)]
pub(super) struct ColumnBounds(BTreeMap<String, BTreeSet<String>>);

impl ColumnBounds {
    pub(super) fn compile(
        configured: &BTreeMap<String, ColumnBound>,
        catalog: &SchemaCatalog,
    ) -> Result<Self> {
        let mut resolved: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (name, bound) in configured {
            if bound.reason.trim().is_empty() {
                bail!("{RULE_ID} option columnBounds: {name} needs a reason");
            }
            let (relation, column) = split(name).ok_or_else(|| {
                anyhow::anyhow!("{RULE_ID} option columnBounds: {name} must name table.column")
            })?;
            let Some(table) = catalog.bounded_relation(&relation) else {
                bail!(
                    "{RULE_ID} option columnBounds: {name} does not resolve to one catalog table"
                );
            };
            if !table
                .columns
                .iter()
                .any(|candidate| candidate.name == column)
            {
                bail!("{RULE_ID} option columnBounds: {name} does not resolve to a catalog column");
            }
            if !resolved
                .entry(table.name.clone())
                .or_default()
                .insert(column)
            {
                bail!(
                    "{RULE_ID} option columnBounds: {name} resolves to a duplicate catalog column"
                );
            }
        }
        Ok(Self(resolved))
    }

    pub(super) fn columns<'a>(&'a self, table: &str) -> impl Iterator<Item = &'a str> {
        self.0.get(table).into_iter().flatten().map(String::as_str)
    }
}

fn split(value: &str) -> Option<(String, String)> {
    let mut parser = Parser::new(&PostgreSqlDialect {})
        .try_with_sql(value)
        .ok()?;
    let mut name = parser.parse_object_name(false).ok()?;
    if name.0.len() < 2 || !matches!(parser.peek_token().token, Token::EOF) {
        return None;
    }
    let column = ident_key(name.0.pop()?.as_ident()?);
    Some((name.to_string(), column))
}
