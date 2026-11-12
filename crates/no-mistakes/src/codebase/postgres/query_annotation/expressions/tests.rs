use super::{expression, Expr};
use oxc_ast::ast::Expression;
use oxc_ast_visit::{walk, Visit};
use std::path::PathBuf;

#[test]
fn dynamic_import_keeps_specifier_and_option_effects_opaque() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/query-annotation-dynamic-import.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    struct Imports<'s> {
        source: &'s str,
        values: Vec<Expr>,
    }
    impl<'a> Visit<'a> for Imports<'_> {
        fn visit_expression(&mut self, value: &Expression<'a>) {
            if matches!(value, Expression::ImportExpression(_)) {
                self.values.push(expression(value, self.source));
            }
            walk::walk_expression(self, value);
        }
    }
    let values = crate::ast::with_program(&path, &source, |program, _| {
        let mut imports = Imports {
            source: &source,
            values: Vec::new(),
        };
        imports.visit_program(program);
        imports.values
    })
    .unwrap();
    assert_eq!(values.len(), 3);
    assert_eq!(values[0], Expr::ModuleLoad("./plugin.mjs".into()));
    assert_eq!(
        values[1],
        Expr::Opaque(vec![Expr::Name("moduleName".into())])
    );
    let Expr::Opaque(children) = &values[2] else {
        panic!("dynamic import must retain opaque child expressions");
    };
    assert_eq!(children.len(), 2);
    let resolver_args = call(&children[0], "resolvePlugin", &source);
    assert_eq!(resolver_args.len(), 1);
    assert_eq!(
        call(&resolver_args[0], "runQuery", &source),
        &[Expr::Text("SELECT module_name FROM plugins".into())]
    );
    let Expr::Record(options, names) = &children[1] else {
        panic!("import options must retain their effects");
    };
    assert_eq!(names, &["with"]);
    let Expr::Record(attributes, names) = &options[0] else {
        panic!("import attributes must retain their effects");
    };
    assert_eq!(names, &["type"]);
    assert_eq!(
        call(&attributes[0], "noteImport", &source),
        &[Expr::Text("json".into())]
    );
}

fn call<'a>(value: &'a Expr, name: &str, source: &str) -> &'a [Expr] {
    let Expr::Call {
        callee,
        args,
        start,
        end,
    } = value
    else {
        panic!("expected preserved {name} call");
    };
    assert_eq!(callee.as_ref(), &Expr::Name(name.into()));
    assert!(source[*start as usize..*end as usize].starts_with(name));
    args
}
