//! Explicit schema membership narrows physical DDL without guessing resolution.
use super::State;
use crate::codebase::postgres::decoded_parts;
use sqlparser::ast::{Expr, Value};
use std::collections::BTreeSet;

pub(super) type Snapshot = (bool, Option<Vec<String>>, Option<Vec<String>>);

impl State {
    pub fn record_path(&mut self, values: &[Expr], path: &Option<Vec<String>>) {
        // The existing quoted-list parser is deliberately conservative here: commas
        // inside quoted schemas and role substitution need a broader GUC parser.
        let known = values.iter().all(|value| match value {
            Expr::Identifier(ident) => {
                ident.value != "$user"
                    && (ident.quote_style.is_some() || !ident.value.eq_ignore_ascii_case("default"))
            }
            Expr::Value(value) => matches!(&value.value, Value::SingleQuotedString(raw)
                if !raw.contains('"') && !raw.contains('$')),
            _ => false,
        });
        self.search_path = path
            .as_ref()
            .filter(|parts| known && parts.iter().all(|part| !part.is_empty()))
            .cloned();
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
