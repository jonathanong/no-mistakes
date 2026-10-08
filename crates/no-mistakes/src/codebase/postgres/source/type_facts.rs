use super::{columns::column, expressions::name, locations::Locations, types::*};
use sqlparser::ast::{ArrayElemTypeDef, DataType};

pub(super) fn data_type(value: &DataType, locations: &Locations<'_>) -> PostgresSqlType {
    let mut result = PostgresSqlType {
        sql: value.to_string(),
        name: None,
        builtin: None,
        modifiers: Vec::new(),
        array_dimensions: Vec::new(),
        fields: Vec::new(),
    };
    match value {
        DataType::Array(
            ArrayElemTypeDef::SquareBracket(inner, dimension)
            | ArrayElemTypeDef::Qualified(inner, dimension),
        ) => {
            result = data_type(inner, locations);
            result.sql = value.to_string();
            result.array_dimensions.push(*dimension);
        }
        DataType::Custom(custom, modifiers) => {
            result.name = Some(name(custom));
            result.modifiers = modifiers.clone();
        }
        DataType::Table(Some(fields)) => {
            result.fields = fields
                .iter()
                .map(|field| column(field, locations, Vec::new()))
                .collect()
        }
        _ => {
            result.modifiers = modifiers(value);
            result.builtin = Some(value.to_string().to_ascii_lowercase());
        }
    }
    result
}

fn modifiers(value: &DataType) -> Vec<String> {
    use sqlparser::ast::ExactNumberInfo;
    match value {
        DataType::Numeric(info) | DataType::Decimal(info) | DataType::Dec(info) => match info {
            ExactNumberInfo::None => Vec::new(),
            ExactNumberInfo::Precision(precision) => vec![precision.to_string()],
            ExactNumberInfo::PrecisionAndScale(precision, scale) => {
                vec![precision.to_string(), scale.to_string()]
            }
        },
        DataType::Varchar(length)
        | DataType::CharacterVarying(length)
        | DataType::Char(length)
        | DataType::Character(length) => length.iter().map(ToString::to_string).collect(),
        DataType::Time(precision, _) | DataType::Timestamp(precision, _) => {
            precision.iter().map(ToString::to_string).collect()
        }
        _ => Vec::new(),
    }
}
