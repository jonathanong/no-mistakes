use super::static_index;
use oxc_ast::ast::ComputedMemberExpression;
use oxc_ast_visit::{walk, Visit};
use oxc_span::GetSpan;
use std::path::PathBuf;

#[test]
fn saved_unary_indices_normalize_only_pure_numeric_literals() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-unary-argument-index/src/query.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    struct Indices<'s> {
        source: &'s str,
        values: Vec<(String, Option<usize>)>,
    }
    impl<'a> Visit<'a> for Indices<'_> {
        fn visit_computed_member_expression(&mut self, value: &ComputedMemberExpression<'a>) {
            let span = value.expression.span();
            let text = &self.source[span.start as usize..span.end as usize];
            self.values
                .push((text.trim().to_string(), static_index(&value.expression)));
            walk::walk_computed_member_expression(self, value);
        }
    }
    let values = crate::ast::with_program(&path, &source, |program, _| {
        let mut indices = Indices {
            source: &source,
            values: Vec::new(),
        };
        indices.visit_program(program);
        indices.values
    })
    .unwrap();
    for value in ["+0", "-0", "+(+0)", "-(-0)", "\"0\""] {
        assert!(
            values.contains(&(value.to_string(), Some(0))),
            "{value}: {values:?}"
        );
    }
    for value in [
        "-1",
        "1.5",
        "9007199254740992",
        "\"01\"",
        "\"extra\"",
        "+key",
        "+(+key)",
        "+!key",
        "key",
        "!key",
    ] {
        assert!(
            values.contains(&(value.to_string(), None)),
            "{value}: {values:?}"
        );
    }
}
