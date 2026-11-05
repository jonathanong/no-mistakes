use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{ObjectName, TableConstraint};

pub(in crate::codebase::postgres::migration) fn constraint_name(
    constraint: &TableConstraint,
) -> Option<String> {
    match constraint {
        TableConstraint::ForeignKey(fk) => fk.name.as_ref().map(ident_key),
        TableConstraint::Check(check) => check.name.as_ref().map(ident_key),
        TableConstraint::Unique(unique) => unique.name.as_ref().map(ident_key),
        TableConstraint::PrimaryKey(primary) => primary.name.as_ref().map(ident_key),
        _ => None,
    }
}

pub(super) fn table_key(name: &ObjectName) -> String {
    crate::codebase::postgres::idents::object_name_identity(name)
}

pub(super) fn table_unqualified_key(name: &ObjectName) -> String {
    crate::codebase::postgres::idents::object_name_ident(name)
        .map(ident_key)
        .unwrap_or_default()
}

pub(super) fn default_check_name(table: &str, constraint: &TableConstraint) -> Option<String> {
    let TableConstraint::Check(check) = constraint else {
        return None;
    };
    let columns = crate::codebase::postgres::idents::collect_ident_names(&check.expr);
    (columns.len() == 1).then(|| format!("{table}_{}_check", columns[0]))
}
