use super::function_call_tests::{
    bare_name, fixture, parsed, projected_function, select_of, NamedArguments,
};
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArg, FunctionArgExpr, FunctionArgOperator, FunctionArguments,
    SelectItem, Statement, TableFactor, Visit, Visitor,
};
use std::ops::ControlFlow;

#[test]
fn legacy_assignment_keeps_non_identifier_assignments() {
    let statements = parsed(fixture("shape"));
    assert!(matches!(
        select_of(&statements[0]).projection[0],
        SelectItem::UnnamedExpr(Expr::Nested(ref inner))
            if matches!(inner.as_ref(), Expr::BinaryOp { op: BinaryOperator::Assignment, .. })
    ));
    let FunctionArguments::List(callee) = &projected_function(&statements[1]).args else {
        panic!("expected callee arguments");
    };
    assert!(
        matches!(
            &callee.args[0],
            FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Nested(inner)))
                if matches!(inner.as_ref(), Expr::BinaryOp { op: BinaryOperator::Assignment, .. })
        ),
        "lower(x) := y stays an assignment expression"
    );
    let mut named = NamedArguments {
        arrow_secs: false,
        assignment_secs: false,
    };
    let _ = statements[2].visit(&mut named);
    assert!(
        named.assignment_secs,
        "parenthesized secs := value is named"
    );
    let mut nested = AssignmentNames {
        outer: false,
        inner: false,
    };
    let _ = statements[3].visit(&mut nested);
    assert!(
        nested.outer && nested.inner,
        "nested := arguments are both named"
    );
    let Statement::Update(update) = &statements[4] else {
        panic!("expected an update assignment");
    };
    assert_eq!(update.assignments.len(), 1);
    let TableFactor::Table {
        args: Some(args), ..
    } = &select_of(&statements[5]).from[0].relation
    else {
        panic!("expected a table function");
    };
    assert!(
        matches!(
            &args.args[0],
            FunctionArg::ExprNamed {
                operator: FunctionArgOperator::Assignment,
                ..
            }
        ),
        "table-function := arguments are named"
    );
}

struct AssignmentNames {
    outer: bool,
    inner: bool,
}

impl Visitor for AssignmentNames {
    type Break = ();

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<Self::Break> {
        let Expr::Function(function) = expr else {
            return ControlFlow::Continue(());
        };
        let FunctionArguments::List(list) = &function.args else {
            return ControlFlow::Continue(());
        };
        let named = list.args.iter().any(|arg| {
            matches!(
                arg,
                FunctionArg::ExprNamed {
                    operator: FunctionArgOperator::Assignment,
                    ..
                }
            )
        });
        if bare_name(function).eq_ignore_ascii_case("make_interval") {
            self.outer = named;
        }
        if bare_name(function).eq_ignore_ascii_case("other") {
            self.inner = named;
        }
        ControlFlow::Continue(())
    }
}
