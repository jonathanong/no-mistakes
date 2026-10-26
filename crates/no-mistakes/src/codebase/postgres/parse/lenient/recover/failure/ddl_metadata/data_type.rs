//! sqlparser accepts some MySQL numeric syntax even with PostgreSqlDialect.
//! Reject those modifiers, preserving bare/custom names such as `unsigned`.
use sqlparser::ast::{ArrayElemTypeDef, DataType, ExactNumberInfo};
pub(super) fn valid(data_type: &DataType) -> bool {
    match data_type {
        DataType::DecimalUnsigned(_)
        | DataType::DecUnsigned(_)
        | DataType::FloatUnsigned(_)
        | DataType::TinyIntUnsigned(_)
        | DataType::Int2Unsigned(_)
        | DataType::SmallIntUnsigned(_)
        | DataType::MediumIntUnsigned(_)
        | DataType::IntUnsigned(_)
        | DataType::Int4Unsigned(_)
        | DataType::IntegerUnsigned(_)
        | DataType::BigIntUnsigned(_)
        | DataType::Int8Unsigned(_)
        | DataType::RealUnsigned
        | DataType::DoubleUnsigned(_)
        | DataType::DoublePrecisionUnsigned
        | DataType::SignedInteger
        | DataType::UnsignedInteger
        | DataType::Int(Some(_))
        | DataType::Integer(Some(_))
        | DataType::SmallInt(Some(_))
        | DataType::BigInt(Some(_))
        | DataType::Float(ExactNumberInfo::PrecisionAndScale(_, _)) => false,
        DataType::Array(array) => match array {
            ArrayElemTypeDef::SquareBracket(element, _)
            | ArrayElemTypeDef::Qualified(element, _) => valid(element),
            _ => false,
        },
        _ => true,
    }
}
