use sqlparser::ast::{Expr, ObjectName, Query, TableFactor};

pub(crate) trait Projection {
    fn table_arm(&self) {}

    #[inline(always)]
    fn inspect_factor(&self, _: &TableFactor) {}

    #[inline(always)]
    fn inspect_query(&self, _: &Query) {}

    #[inline(always)]
    fn relation_name(&self, _: &ObjectName, original: String) -> String {
        original
    }

    #[inline(always)]
    fn qualifier_matches(&self, table: &str, qualifier: &str) -> bool {
        table.rsplit('.').next() == Some(qualifier)
    }

    fn map<'a>(
        &self,
        predicate: &Expr,
        other: Option<&Expr>,
        table: &str,
        column: String,
        proven: bool,
        peers: impl Iterator<Item = &'a str>,
    ) -> Option<(String, bool)>;
}

pub(crate) struct Identity;

impl Projection for Identity {
    #[inline(always)]
    fn map<'a>(
        &self,
        _: &Expr,
        _: Option<&Expr>,
        _: &str,
        column: String,
        proven: bool,
        _: impl Iterator<Item = &'a str>,
    ) -> Option<(String, bool)> {
        Some((column, proven))
    }
}
