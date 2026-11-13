use oxc_ast::ast::{BindingPattern, ForOfStatement, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};

#[test]
fn cov_p2_finite_names_reject_nonliteral_rows_and_builder_prefixes() {
    struct Grammar {
        loops: Vec<Option<Vec<String>>>,
        roots: Vec<(String, bool)>,
    }
    impl<'a> Visit<'a> for Grammar {
        fn visit_for_of_statement(&mut self, value: &ForOfStatement<'a>) {
            self.loops
                .push(super::loop_domain(value).map(|domain| domain.values));
            walk::walk_for_of_statement(self, value);
        }
        fn visit_variable_declarator(&mut self, value: &VariableDeclarator<'a>) {
            if let (BindingPattern::BindingIdentifier(id), Some(init)) = (&value.id, &value.init) {
                if id.name.starts_with("yes") || id.name.starts_with("no") {
                    let root = super::root((id.span.start, id.span.end), (0, 0), init);
                    self.roots.push((id.name.to_string(), root.is_some()));
                }
            }
            walk::walk_variable_declarator(self, value);
        }
    }
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("cov-p2-import-gate/grammar.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    crate::ast::with_program(&path, &source, |program, _| {
        let mut grammar = Grammar {
            loops: Vec::new(),
            roots: Vec::new(),
        };
        grammar.visit_program(program);
        assert_eq!(grammar.loops.len(), 18);
        for (index, domain) in grammar.loops.iter().enumerate() {
            let expected = matches!(index, 0 | 11).then(|| vec!["first".into(), "second".into()]);
            assert_eq!(*domain, expected, "loop {index}");
        }
        assert_eq!(grammar.roots.len(), 8);
        for (name, accepted) in grammar.roots {
            assert_eq!(accepted, name.starts_with("yes"), "{name}");
        }
    })
    .unwrap();
}
