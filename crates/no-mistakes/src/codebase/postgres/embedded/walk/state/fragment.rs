use super::ScopeVisitor;
use crate::codebase::postgres::embedded::EmbeddedSqlFragment;

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn push_fragment(
        &mut self,
        line: u32,
        sql_text: Option<String>,
        expression: Option<&oxc_ast::ast::Expression<'_>>,
        append_site: Option<u32>,
    ) {
        let variants = expression
            .and_then(|expression| self.recover_variants(expression))
            .filter(|values| {
                values
                    .iter()
                    .all(|value| value.value == super::super::variants::ValueKind::Sql)
            })
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.publish())
            .collect();
        self.fragment_variants.push(variants);
        self.fragment_sites.push(append_site);
        self.fragments.push(EmbeddedSqlFragment {
            line,
            sql_text,
            recovered_placeholder_positions: Vec::new(),
        });
    }
}
