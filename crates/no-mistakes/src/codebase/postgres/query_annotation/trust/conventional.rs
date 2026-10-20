use oxc_ast::ast::Program;
use std::collections::BTreeSet;

/// An identifier spelled `sql` is not a trusted tag. Callers insert a name
/// only when an import resolves to `sql-template-strings` or `trustedSqlTags`.
pub(super) fn collect(_program: &Program<'_>) -> BTreeSet<String> {
    BTreeSet::new()
}
