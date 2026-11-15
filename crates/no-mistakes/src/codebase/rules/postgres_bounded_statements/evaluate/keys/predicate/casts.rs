//! A cast matches a partial-index literal only when the value cannot change.
use super::shape::Lit;
use crate::codebase::postgres::idents::object_name_key;
use crate::codebase::postgres::SchemaCatalog;
use sqlparser::ast::{ArrayElemTypeDef, CharLengthUnits, CharacterLength, DataType, ObjectName};

pub(super) fn preserves(data_type: &DataType, lit: &Lit, catalog: &SchemaCatalog) -> bool {
    match data_type {
        DataType::Array(
            ArrayElemTypeDef::SquareBracket(inner, _) | ArrayElemTypeDef::Qualified(inner, _),
        ) => preserves(inner, lit, catalog),
        DataType::Text => matches!(lit, Lit::Str(_)),
        DataType::Boolean => matches!(lit, Lit::Bool(_)),
        DataType::Varchar(length)
        | DataType::CharacterVarying(length)
        | DataType::Nvarchar(length)
        | DataType::CharVarying(length) => string_fits(lit, length),
        DataType::Custom(name, modifiers) if modifiers.is_empty() => enum_label(name, lit, catalog),
        _ => false,
    }
}

fn string_fits(lit: &Lit, length: &Option<CharacterLength>) -> bool {
    let Lit::Str(text) = lit else {
        return false;
    };
    let Some(limit) = character_limit(length) else {
        return false;
    };
    text.chars().count() as u64 <= limit
}

fn character_limit(length: &Option<CharacterLength>) -> Option<u64> {
    match length {
        None => Some(u64::MAX),
        Some(CharacterLength::Max) => Some(u64::MAX),
        Some(CharacterLength::IntegerLength {
            length,
            unit: None | Some(CharLengthUnits::Characters),
        }) => Some(*length),
        Some(CharacterLength::IntegerLength {
            unit: Some(CharLengthUnits::Octets),
            ..
        }) => None,
    }
}

fn enum_label(name: &ObjectName, lit: &Lit, catalog: &SchemaCatalog) -> bool {
    let Lit::Str(text) = lit else {
        return false;
    };
    catalog
        .enum_type(&object_name_key(name))
        .is_some_and(|enum_type| enum_type.values.iter().any(|label| label == text))
}

#[cfg(test)]
#[path = "casts_tests.rs"]
mod casts_tests;
