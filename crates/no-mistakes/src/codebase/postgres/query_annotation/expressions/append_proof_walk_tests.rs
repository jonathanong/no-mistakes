use super::super::super::proof_walk;
use super::function_expression;
use oxc_ast::ast::{Declaration, Program, Statement};

#[test]
fn append_callback_census_runs_only_when_the_proof_can_change() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/append-callback-proof-walk.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let walks = crate::ast::with_program(&path, &source, |program, source| {
        (
            proof(program, source, "listWidgets"),
            proof(program, source, "listItems"),
            proof(program, source, "listOrders"),
            proof(program, source, "listShipments"),
        )
    })
    .unwrap();
    assert!(walks.0.proven, "conditional append still proves");
    assert_eq!(walks.0.walks, 1, "a live proof still walks once");
    assert!(!walks.1.proven, "a call after the callback drops the proof");
    assert_eq!(walks.1.walks, 0, "a rejected result read does not walk");
    assert!(!walks.2.proven);
    assert_eq!(
        walks.2.walks, 0,
        "an unsupported helper with no callback does not re-lower"
    );
    assert!(!walks.3.proven);
    assert_eq!(
        walks.3.walks, 0,
        "a branch without a callback call does not walk"
    );
}

struct Walk {
    proven: bool,
    walks: u32,
}

fn proof(program: &Program<'_>, source: &str, name: &str) -> Walk {
    let before = proof_walk::count();
    let function = function(program, name);
    let super::super::Expr::Function(summary) = function_expression(function, source) else {
        panic!("{name} did not lower to a function");
    };
    Walk {
        proven: summary.append_callback.is_some(),
        walks: proof_walk::count() - before,
    }
}

fn function<'a>(program: &'a Program<'a>, name: &str) -> &'a oxc_ast::ast::Function<'a> {
    program
        .body
        .iter()
        .find_map(|statement| {
            let function = match statement {
                Statement::FunctionDeclaration(function) => function,
                Statement::ExportDeclaration(export) => match &export.declaration {
                    Declaration::FunctionDeclaration(function) => function,
                    _ => return None,
                },
                _ => return None,
            };
            function
                .id
                .as_ref()
                .is_some_and(|id| id.name == name)
                .then_some(function.as_ref())
        })
        .unwrap_or_else(|| panic!("missing {name}"))
}
