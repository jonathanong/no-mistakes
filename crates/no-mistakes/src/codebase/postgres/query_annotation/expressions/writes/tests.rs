use super::*;
use oxc_ast_visit::walk;

#[test]
fn opaque_write_targets_exclude_receivers_defaults_and_keep_ts_wrapped_bindings() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-opaque-write-targets/targets.ts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    struct Writes<'s> {
        source: &'s str,
        targets: Vec<Vec<String>>,
    }
    impl<'a> Visit<'a> for Writes<'_> {
        fn visit_expression(&mut self, expression: &Expression<'a>) {
            if matches!(
                expression,
                Expression::AssignmentExpression(_) | Expression::UpdateExpression(_)
            ) {
                let Expr::OpaqueWrite { targets, .. } = collect(expression, self.source) else {
                    panic!("opaque write");
                };
                self.targets.push(targets);
            }
            walk::walk_expression(self, expression);
        }
    }
    let targets = crate::ast::with_program(&path, &source, |program, _| {
        let mut writes = Writes {
            source: &source,
            targets: Vec::new(),
        };
        writes.visit_program(program);
        writes.targets
    })
    .unwrap();
    let expected = vec![
        vec!["direct"],
        vec!["updated"],
        vec!["arrayTarget", "restTarget"],
        vec!["objectTarget"],
        vec![],
        vec![],
        vec!["asTarget"],
        vec!["satisfiesTarget"],
        vec!["nonNullTarget"],
        vec!["assertedTarget"],
        vec![],
        vec!["direct"],
        vec![],
    ];
    assert_eq!(targets, expected);
}
