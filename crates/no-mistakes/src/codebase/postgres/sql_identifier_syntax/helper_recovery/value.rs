//! Values preserve physical provenance when crossing a helper boundary.
use super::sql_text::SqlText;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Value {
    Scalar,
    OpaqueMember,
    Constrained { truth: bool, executor_payload: bool },
    Executor,
    MaybeExecutor,
    Promise(Box<Value>),
    Undefined,
    Null,
    Bool(bool),
    Text(SqlText),
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    Builder(usize),
    Sql(SqlText),
}
impl Value {
    pub fn truth(&self) -> Option<bool> {
        match self {
            Self::Scalar | Self::OpaqueMember | Self::MaybeExecutor => None,
            Self::Constrained { truth, .. } => Some(*truth),
            Self::Undefined | Self::Null => Some(false),
            Self::Bool(value) => Some(*value),
            Self::Text(value) => Some(!value.raw.sql.is_empty()),
            _ => Some(true),
        }
    }
    pub fn member(&self, name: &str) -> Option<Self> {
        match self {
            Self::Object(props) => match props.iter().rev().find(|(key, _)| key == name) {
                Some((_, value)) => Some(value.clone()),
                None if inherited(name) => None,
                None => Some(Self::Undefined),
            },
            Self::Scalar => Some(Self::Scalar),
            Self::OpaqueMember => Some(Self::OpaqueMember),
            Self::Constrained {
                executor_payload, ..
            } => Some(if *executor_payload {
                Self::OpaqueMember
            } else {
                Self::Scalar
            }),
            Self::Executor => Some(if name == "query" {
                Self::MaybeExecutor
            } else {
                Self::OpaqueMember
            }),
            // Nullish receiver evaluation throws rather than exposing a value.
            _ => None,
        }
    }
    pub fn cache_hash(&self, hash: &mut impl Hasher) {
        std::mem::discriminant(self).hash(hash);
        match self {
            Self::Bool(value) => value.hash(hash),
            Self::Constrained {
                truth,
                executor_payload,
            } => (truth, executor_payload).hash(hash),
            Self::Promise(value) => value.cache_hash(hash),
            Self::Builder(id) => id.hash(hash),
            Self::Text(text) | Self::Sql(text) => {
                text.binds.hash(hash);
                text.executor_payload.hash(hash);
                let text = &text.raw;
                text.sql.hash(hash);
                text.line.hash(hash);
                text.origins.hash(hash);
                for pos in &text.positions {
                    (
                        pos.sql_line,
                        pos.sql_column,
                        pos.source_line,
                        &pos.source_path,
                    )
                        .hash(hash);
                }
            }
            Self::Object(props) => {
                for (key, value) in props {
                    key.hash(hash);
                    value.cache_hash(hash);
                }
            }
            Self::Array(items) => {
                for item in items {
                    item.cache_hash(hash);
                }
            }
            _ => {}
        }
    }
    pub fn contains_executor(&self) -> bool {
        match self {
            Self::Executor | Self::MaybeExecutor | Self::OpaqueMember => true,
            Self::Constrained {
                executor_payload, ..
            } => *executor_payload,
            Self::Promise(value) => value.contains_executor(),
            Self::Text(text) | Self::Sql(text) => text.executor_payload,
            Self::Object(props) => props.iter().any(|(_, value)| value.contains_executor()),
            Self::Array(items) => items.iter().any(Self::contains_executor),
            _ => false,
        }
    }
    pub fn bytes(&self) -> usize {
        match self {
            Self::Text(text) | Self::Sql(text) => text.raw.sql.len(),
            Self::Promise(value) => value.bytes(),
            Self::Object(props) => props.iter().map(|(_, value)| value.bytes()).sum(),
            Self::Array(items) => items.iter().map(Self::bytes).sum(),
            _ => 0,
        }
    }
    pub fn read_only_argument(&self) -> bool {
        match self {
            Self::Sql(_) => true,
            Self::Promise(value) => value.read_only_argument(),
            Self::Object(props) => props.iter().all(|(_, value)| value.read_only_argument()),
            Self::Array(items) => items.iter().all(Self::read_only_argument),
            other => other.owned_argument(),
        }
    }
    pub fn owned_argument(&self) -> bool {
        match self {
            Self::Builder(_) | Self::Sql(_) => false,
            Self::Promise(value) => value.owned_argument(),
            Self::Object(props) => props.iter().all(|(_, value)| value.owned_argument()),
            Self::Array(items) => items.iter().all(Self::owned_argument),
            _ => true,
        }
    }
}

// These names are inherited on ordinary fresh records, not absent option fields.
// Decline the read rather than inventing prototype/coercion semantics.
fn inherited(name: &str) -> bool {
    matches!(
        name,
        "constructor"
            | "toString"
            | "toLocaleString"
            | "valueOf"
            | "hasOwnProperty"
            | "isPrototypeOf"
            | "propertyIsEnumerable"
            | "__proto__"
            | "__defineGetter__"
            | "__defineSetter__"
            | "__lookupGetter__"
            | "__lookupSetter__"
    )
}
