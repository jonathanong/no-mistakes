use super::shadows_set;
use oxc_ast::ast::{Function, Statement};

fn function<'a>(program: &'a oxc_ast::ast::Program<'a>, name: &str) -> &'a Function<'a> {
    program
        .body
        .iter()
        .find_map(|statement| {
            let Statement::FunctionDeclaration(function) = statement else {
                return None;
            };
            function
                .id
                .as_ref()
                .is_some_and(|id| id.name == name)
                .then_some(function.as_ref())
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn saved_fixture_rejects_set_parameters_and_catches() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture/fail-shadowed-set-guard/src/query.ts"
    ));
    crate::ast::with_program(std::path::Path::new("query.ts"), source, |program, _| {
        for (name, shadowed) in [("trusted", false), ("shadowed", true), ("caught", true)] {
            let function = function(program, name);
            let body = function.body.as_ref().unwrap();
            assert_eq!(shadows_set(&function.params, body), shadowed, "{name}");
        }
    })
    .unwrap();
}
