use super::tables::CONSTRUCTORS;
use super::values;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ValueKind {
    String,
    Number,
    Boolean,
    BigInt,
    Array {
        element: Option<PrimitiveKind>,
        empty: bool,
    },
    Promise,
    AsyncStorage,
    Builtin,
    Namespace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PrimitiveKind {
    String,
    Number,
    Boolean,
    BigInt,
}

impl PrimitiveKind {
    pub(super) fn from_value(value: ValueKind) -> Option<Self> {
        match value {
            ValueKind::String => Some(Self::String),
            ValueKind::Number => Some(Self::Number),
            ValueKind::Boolean => Some(Self::Boolean),
            ValueKind::BigInt => Some(Self::BigInt),
            _ => None,
        }
    }
}

#[derive(Default)]
pub(super) struct Names {
    pub(super) map: FxHashMap<String, Vec<Option<ValueKind>>>,
    pub(super) properties: FxHashMap<String, FxHashMap<String, Vec<Option<ValueKind>>>>,
    pub(super) storage_constructors: FxHashSet<String>,
}

impl Names {
    pub(super) fn type_kind(&self, ty: &TSType<'_>) -> Option<ValueKind> {
        match ty {
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                    return None;
                };
                if self.storage_constructors.contains(name.name.as_str())
                    && self.constructor(&name.name)
                {
                    return Some(ValueKind::AsyncStorage);
                }
                if name.name == "AsyncLocalStorage" || !self.global(&name.name) {
                    return None;
                }
                values::value_kind_from_type(ty)
            }
            TSType::TSParenthesizedType(inner) => self.type_kind(&inner.type_annotation),
            TSType::TSTypeOperatorType(inner) => self.type_kind(&inner.type_annotation),
            TSType::TSUnionType(union) => {
                let kinds = union
                    .types
                    .iter()
                    .map(|ty| self.type_kind(ty))
                    .collect::<Option<Vec<_>>>()?;
                let first = *kinds.first()?;
                Some(if kinds.iter().all(|kind| *kind == first) {
                    first
                } else if kinds
                    .iter()
                    .all(|kind| matches!(kind, ValueKind::Array { .. }))
                {
                    ValueKind::Array {
                        element: None,
                        empty: false,
                    }
                } else {
                    ValueKind::Builtin
                })
            }
            _ => values::value_kind_from_type(ty),
        }
    }
    pub(super) fn record(&mut self, name: &str, value: Option<ValueKind>) {
        self.map.entry(name.to_owned()).or_default().push(value);
    }

    pub(super) fn record_property(&mut self, name: &str, property: &str, value: Option<ValueKind>) {
        self.properties
            .entry(name.to_owned())
            .or_default()
            .entry(property.to_owned())
            .or_default()
            .push(value);
    }

    pub(super) fn global(&self, name: &str) -> bool {
        !self.map.contains_key(name)
    }

    pub(super) fn constructor(&self, name: &str) -> bool {
        if self.storage_constructors.contains(name) {
            return self.map.get(name).is_some_and(|all| all.len() == 1);
        }
        name != "AsyncLocalStorage" && CONSTRUCTORS.contains(&name) && self.global(name)
    }
}
