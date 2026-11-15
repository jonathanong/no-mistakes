//! PostgreSQL call arguments written `name := value`.
//!
//! With expression names enabled, the stock parser accepts `=>` as a named
//! argument and consumes `:=` as an assignment expression before it can see a
//! named-argument operator. Rewrite only a call argument whose left side is an
//! identifier. `UPDATE` assignments and `lower(x) := y` stay assignment
//! expressions.
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArg, FunctionArgExpr, FunctionArgOperator, FunctionArguments,
    Ident, TableFactor, Value, VisitMut, VisitorMut,
};
use std::ops::ControlFlow;

pub(super) fn rewrite(tree: &mut impl VisitMut) {
    let _ = tree.visit(&mut Rewrite);
}

struct Rewrite;

impl VisitorMut for Rewrite {
    type Break = ();

    fn pre_visit_table_factor(&mut self, factor: &mut TableFactor) -> ControlFlow<()> {
        match factor {
            TableFactor::Function { args, .. } => {
                for arg in args {
                    rewrite_argument(arg);
                }
            }
            TableFactor::Table {
                args: Some(args), ..
            } => {
                for arg in &mut args.args {
                    rewrite_argument(arg);
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }

    fn post_visit_expr(&mut self, expr: &mut Expr) -> ControlFlow<()> {
        if let Expr::Function(function) = expr {
            rewrite_arguments(&mut function.parameters);
            rewrite_arguments(&mut function.args);
        }
        ControlFlow::Continue(())
    }
}

fn rewrite_arguments(args: &mut FunctionArguments) {
    let FunctionArguments::List(list) = args else {
        return;
    };
    for arg in &mut list.args {
        rewrite_argument(arg);
    }
}

fn rewrite_argument(arg: &mut FunctionArg) {
    let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg else {
        return;
    };
    let taken = std::mem::replace(expr, Expr::Value(Value::Null.into()));
    match named_assignment(taken) {
        Ok((name, value)) => {
            *arg = FunctionArg::ExprNamed {
                name: Expr::Identifier(name),
                arg: FunctionArgExpr::Expr(value),
                operator: FunctionArgOperator::Assignment,
            };
        }
        Err(original) => {
            if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                *expr = *original;
            }
        }
    }
}

fn named_assignment(expr: Expr) -> Result<(Ident, Expr), Box<Expr>> {
    match expr {
        Expr::Nested(inner) => match named_assignment(*inner) {
            Ok(named) => Ok(named),
            Err(inner) => Err(Box::new(Expr::Nested(inner))),
        },
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Assignment,
            right,
        } => match *left {
            Expr::Identifier(name) => Ok((name, *right)),
            left => Err(Box::new(Expr::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Assignment,
                right,
            })),
        },
        other => Err(Box::new(other)),
    }
}
