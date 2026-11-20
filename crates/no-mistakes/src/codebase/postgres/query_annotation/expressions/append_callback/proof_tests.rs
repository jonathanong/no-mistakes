use super::{destructured_append, inert_after_call, result_read};
use crate::ast::with_program;
use crate::codebase::postgres::query_annotation::statements::destructured::RESULT;
use crate::codebase::postgres::query_annotation::{self, Expr, Function, Step};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::sync::Arc;

fn saved_helpers() -> Vec<(String, Arc<Function>)> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/imported-callback-append-branchless/pager.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("./db.mjs", &[]);
    with_program(&path, &source, |program, _| {
        let facts = query_annotation::collect(program, &source, &options);
        let mut found = Vec::new();
        hoisted(&facts.roots, &mut found);
        found.sort_by(|left, right| left.0.cmp(&right.0));
        found
    })
    .unwrap()
}

fn hoisted(steps: &[Step], found: &mut Vec<(String, Arc<Function>)>) {
    for step in steps {
        match step {
            Step::Hoisted(name, Expr::Function(function)) => {
                found.push((name.clone(), Arc::clone(function)));
            }
            Step::Branch(_, left, right) => {
                hoisted(left, found);
                hoisted(right, found);
            }
            _ => {}
        }
    }
}

fn helper<'a>(helpers: &'a [(String, Arc<Function>)], name: &str) -> &'a Function {
    helpers
        .iter()
        .find(|(found, _)| found == name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .1
        .as_ref()
}

fn call_index(function: &Function) -> usize {
    function
        .body
        .iter()
        .rposition(|step| matches!(step, Step::Bind(name, _) if name == RESULT))
        .unwrap_or_else(|| panic!("missing destructured call"))
}

#[test]
fn append_after_the_callback_does_not_widen_the_proof() {
    let helpers = saved_helpers();
    let after = helper(&helpers, "queryItemsAfterCall");
    let site = call_index(after);
    let later = after.body.iter().enumerate().find(|(index, step)| {
        *index > site && matches!(step, Step::Append(name, _) if name == "selectWhere")
    });
    let Some((_, step)) = later else {
        panic!("the saved helper must append after the callback call");
    };
    assert!(destructured_append(after));
    assert!(!inert_after_call(step, "selectWhere"));
    assert!(after.append_callback.is_none());
}

#[test]
fn in_place_destructured_appends_keep_the_proof() {
    let helpers = saved_helpers();
    for name in ["queryIdPage", "queryIdPagePlain", "queryIdPageBranch"] {
        let function = helper(&helpers, name);
        assert!(destructured_append(function), "{name}");
        assert!(function.append_callback.is_some(), "{name}");
    }
    let returned = helper(&helpers, "queryIdPageReturn");
    assert!(!destructured_append(returned));
    assert!(returned.append_callback.is_none());
}

#[test]
fn alias_default_ternary_and_spread_do_not_attach_the_proof() {
    let helpers = saved_helpers();
    for name in [
        "queryItemsAlias",
        "queryItemsDefault",
        "queryItemsTernary",
        "queryItemsSpread",
        "queryItemsReassign",
    ] {
        let function = helper(&helpers, name);
        assert!(function.append_callback.is_none(), "{name}");
    }
    let ternary = helper(&helpers, "queryItemsTernary");
    let Step::Return(ternary_result) = ternary.body.last().unwrap() else {
        panic!("ternary helper must return");
    };
    assert!(!result_read(ternary_result, "selectWhere"));
    let spread = helper(&helpers, "queryItemsSpread");
    let Step::Return(spread_result) = spread.body.last().unwrap() else {
        panic!("spread helper must return");
    };
    assert!(!result_read(spread_result, "selectWhere"));
    assert!(destructured_append(helper(&helpers, "queryItemsAlias")));
    assert!(destructured_append(helper(&helpers, "queryItemsDefault")));
}

#[test]
fn destructured_names_lower_to_primitive() {
    let helpers = saved_helpers();
    for (name, function) in &helpers {
        let mut destructured = false;
        let mut saw_name = false;
        for step in &function.body {
            match step {
                Step::Bind(bound, _) if bound == RESULT => destructured = true,
                Step::Bind(bound, expr) if destructured => {
                    saw_name = true;
                    assert!(
                        matches!(expr, Expr::Primitive),
                        "{name} binds {bound} as {expr:?}"
                    );
                }
                _ => destructured = false,
            }
        }
        if function
            .body
            .iter()
            .any(|step| matches!(step, Step::Bind(bound, _) if bound == RESULT))
        {
            assert!(saw_name, "{name} destructures no names");
        }
    }
}
