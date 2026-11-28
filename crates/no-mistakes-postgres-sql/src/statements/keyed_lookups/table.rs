use sqlparser::ast::{ObjectName, TableAlias, TableFactor, TableInheritance};

pub(super) fn lookup_table(factor: &TableFactor) -> Option<(&ObjectName, Option<&TableAlias>)> {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            sample: None,
            inheritance: None | Some(TableInheritance::IncludeDescendants),
            ..
        } if alias.as_ref().is_none_or(|alias| alias.columns.is_empty()) => {
            Some((name, alias.as_ref()))
        }
        _ => None,
    }
}
