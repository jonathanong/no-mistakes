//! Explicit schema membership narrows physical DDL without guessing resolution.
use super::State;
use crate::codebase::postgres::decoded_parts;
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, Value};
use std::collections::BTreeSet;

pub(super) type Snapshot = (bool, Option<Vec<String>>, Option<Vec<String>>);

impl State {
    pub fn record_path(&mut self, values: &[Expr]) -> Option<Vec<String>> {
        let known = values.iter().all(|value| {
            matches!(value, Expr::Identifier(_))
                || matches!(value, Expr::Value(value) if matches!(value.value, Value::SingleQuotedString(_)))
        });
        let path = values
            .iter()
            .map(path_value)
            .collect::<Option<Vec<_>>>()
            .map(|parts| parts.into_iter().flatten().collect::<Vec<_>>());
        self.search_path = path
            .as_ref()
            .filter(|parts| known && !parts.is_empty())
            .cloned();
        path
    }

    pub fn ddl_names(&self, name: &str) -> BTreeSet<Vec<String>> {
        let parts = decoded_parts(name);
        if let ([relation], Some(path)) = (parts.as_slice(), &self.search_path) {
            // Both implicit namespaces remain eligible even when omitted from SET.
            path.iter()
                .map(String::as_str)
                .chain(["pg_catalog", "pg_temp"])
                .map(|schema| vec![schema.to_owned(), relation.clone()])
                .collect()
        } else {
            BTreeSet::from([parts])
        }
    }
}

fn path_value(value: &Expr) -> Option<Vec<String>> {
    match value {
        Expr::Identifier(ident)
            if ident.value != "$user"
                && (ident.quote_style.is_some()
                    || !ident.value.eq_ignore_ascii_case("default")) =>
        {
            Some(vec![ident_key(ident)])
        }
        Expr::Value(value) => match &value.value {
            // SET parses a quoted SQL value as one schema name, even if it contains
            // commas or double quotes. Only commas between SQL values separate paths.
            Value::SingleQuotedString(raw) if raw.is_empty() => None,
            Value::SingleQuotedString(raw) if !raw.contains('$') => Some(vec![raw.clone()]),
            Value::Number(raw, _) => Some(vec![raw.clone()]),
            _ => None,
        },
        _ => None,
    }
}
