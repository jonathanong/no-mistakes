use sqlparser::ast::{Expr, ObjectName};

pub(crate) trait Projection {
    fn table_arm(&self) {}

    fn nested_join(&self) {}

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
