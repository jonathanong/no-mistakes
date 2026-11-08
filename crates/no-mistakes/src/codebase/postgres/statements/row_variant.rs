use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::ObjectName;

/// Built-in functions that return a different value for each row they are evaluated for, so an
/// equality against one picks out no fixed row. A function that is not listed (a user-defined
/// one included) is assumed to be row-invariant: its volatility is not part of the facts.
#[rustfmt::skip]
const ROW_VARIANT: &[&str] = &[
    "nextval", "currval", "lastval", "setval", "random", "random_normal", "setseed",
    "gen_random_uuid", "uuidv4", "uuidv7", "uuid_generate_v1", "uuid_generate_v1mc",
    "uuid_generate_v4", "clock_timestamp", "timeofday", "pg_sleep",
];

pub(super) fn is_row_variant(name: &ObjectName) -> bool {
    object_name_ident(name).is_some_and(|ident| ROW_VARIANT.contains(&ident_key(ident).as_str()))
}
