use crate::config::source_locations::ConfigPath;
use serde::de::{Deserializer, Visitor};
use serde_yaml::Value;
use std::cell::RefCell;

pub(crate) use no_mistakes_rule_kernel::validate_options::OptionLocation;

use super::RuleDef;

pub(crate) fn invalid_option(
    rule: &RuleDef,
    path: ConfigPath,
    detail: impl std::fmt::Display,
) -> anyhow::Error {
    let name = rule
        .name
        .as_deref()
        .map(|name| format!(" application `{name}`"))
        .unwrap_or_default();
    let message = format!(
        "invalid options for rule `{}`{name} at {path}: {detail}",
        rule.rule
    );
    OptionLocation::semantic(path, message)
}

pub(super) fn option_location_from_error(
    rule: &RuleDef,
    error: serde_path_to_error::Error<serde_yaml::Error>,
    unknown: Option<String>,
) -> anyhow::Error {
    let mut path = ConfigPath::default().key("options");
    for segment in error.path() {
        path = match segment {
            serde_path_to_error::Segment::Map { key } => path.key(key),
            serde_path_to_error::Segment::Seq { index } => path.index(*index),
            serde_path_to_error::Segment::Enum { variant } => path.key(variant),
            serde_path_to_error::Segment::Unknown => break,
        };
    }
    if let Some(key) = &unknown {
        path = path.key(key);
    }
    let serde_path = error.path().to_string();
    let location = match unknown {
        Some(key) => format!("options.{key}"),
        None if serde_path.is_empty() || serde_path == "." => "options".into(),
        None => format!("options.{serde_path}"),
    };
    let name = rule
        .name
        .as_deref()
        .map(|name| format!(" application `{name}`"))
        .unwrap_or_default();
    OptionLocation::semantic(
        path,
        format!(
            "invalid options for rule `{}`{name} at {location}: {}",
            rule.rule,
            error.inner()
        ),
    )
}

pub(super) struct WithKnownFields<'a> {
    pub(super) options: Value,
    pub(super) postgres: bool,
    pub(super) embedded_sql: bool,
    pub(super) allows_unmatched_executors: bool,
    pub(super) unknown: &'a RefCell<Option<String>>,
}

impl<'de> Deserializer<'de> for WithKnownFields<'_> {
    type Error = serde_yaml::Error;

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        if let Some(options) = self.options.as_mapping() {
            for key in options.keys().filter_map(Value::as_str) {
                let known = fields.contains(&key)
                    || (self.postgres && inherited_postgres_key(key))
                    || (self.embedded_sql
                        && matches!(
                            key,
                            "rawSqlTags"
                                | "sqlIdentifierAssertions"
                                | "executorOptionProperties"
                                | "executorOptions"
                        ))
                    || (self.embedded_sql
                        && self.allows_unmatched_executors
                        && key == "reportUnmatchedExecutorNames");
                if !known {
                    *self.unknown.borrow_mut() = Some(key.to_string());
                    return Err(serde::de::Error::unknown_field(key, fields));
                }
            }
        }
        self.options.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.options.deserialize_any(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option
        unit unit_struct newtype_struct seq tuple tuple_struct map enum identifier ignored_any
    }
}

// Shared scope fields can be materialized onto rules whose local option type
// only consumes a subset. The Postgres scope validator owns those fields.
fn inherited_postgres_key(key: &str) -> bool {
    matches!(
        key,
        "importSpecifier"
            | "executorNames"
            | "extraExecutorNames"
            | "excludeExecutorNames"
            | "executorFactoryNames"
            | "executorTypeNames"
            | "trustedSqlTags"
            | "schemaCatalogPath"
            | "unanalyzableSql"
            | "sqlInclude"
            | "sqlIncludeLocked"
    )
}
