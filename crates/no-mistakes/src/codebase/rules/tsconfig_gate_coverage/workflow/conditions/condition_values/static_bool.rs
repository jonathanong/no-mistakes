use super::{StaticBool, StaticValue};

pub(super) fn static_bool_value(value: StaticBool) -> StaticValue {
    match value {
        StaticBool::False => StaticValue::Bool(false),
        StaticBool::True => StaticValue::Bool(true),
        StaticBool::Invalid => StaticValue::Invalid,
        StaticBool::TruthyNonBoolean | StaticBool::Unknown => StaticValue::Unknown,
    }
}
