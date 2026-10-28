use oxc_ast::ast::Function;
use oxc_ast_visit::{walk, Visit};

#[derive(Default)]
struct Candidates(std::collections::BTreeMap<String, bool>);
impl<'a> Visit<'a> for Candidates {
    fn visit_function(&mut self, value: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        if let Some(id) = &value.id {
            self.0.insert(
                id.name.to_string(),
                super::super::finite_returns::function(value).is_some(),
            );
        }
        walk::walk_function(self, value, flags);
    }
}
#[test]
fn terminal_return_collection_keeps_the_full_same_function_return_count() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("terminal-block-returns/src/helpers.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let candidates = crate::ast::with_program(&path, &source, |program, _| {
        let mut candidates = Candidates::default();
        candidates.visit_program(program);
        candidates.0
    })
    .unwrap();
    for name in [
        "named",
        "differentNames",
        "missing",
        "throwsBeforeReturn",
        "independentNestedReturn",
        "switched",
    ] {
        assert!(candidates[name], "{name}");
    }
    for name in ["hiddenReturn", "nestedBranch", "fallthrough"] {
        assert!(!candidates[name], "{name}");
    }
}
