use oxc_ast::ast::{IdentifierReference, Program};
use oxc_ast_visit::Visit;
use std::collections::BTreeSet;

pub(super) fn collect(program: &Program<'_>) -> BTreeSet<String> {
    let mut names = Names(BTreeSet::from(["sql".to_string()]));
    names.visit_program(program);
    names.0
}
struct Names(BTreeSet<String>);
impl<'a> Visit<'a> for Names {
    fn visit_identifier_reference(&mut self, value: &IdentifierReference<'a>) {
        // Preserve the embedded analyzer's case-insensitive conventional tag.
        if value.name.eq_ignore_ascii_case("sql") {
            self.0.insert(value.name.to_string());
        }
    }
}
