use crate::codebase::postgres::statements::SqlBoundQuery;

pub fn query(bound: &SqlBoundQuery) -> String {
    let items: Vec<String> = bound.items.iter().map(super::item).collect();
    let cap = if bound.capped { "capped " } else { "" };
    format!("{cap}{}", items.join(" "))
}
