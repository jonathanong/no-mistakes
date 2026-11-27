use sqlparser::ast::{visit_expressions_mut, BinaryOperator, Expr, ObjectNamePart, Value};
use std::ops::ControlFlow;

pub fn conjuncts_of(expr: &Expr, out: &mut Vec<Expr>) {
    match expr {
        Expr::Nested(inner) => conjuncts_of(inner, out),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            conjuncts_of(left, out);
            conjuncts_of(right, out);
        }
        other => out.push(unnest(other)),
    }
}

pub fn unnest(expr: &Expr) -> Expr {
    match expr {
        Expr::Nested(inner) => unnest(inner),
        other => other.clone(),
    }
}

pub fn canonical_bound_expr(expr: &Expr) -> Expr {
    let mut normalized = unnest(expr);
    let _: ControlFlow<()> = visit_expressions_mut(&mut normalized, |expr| {
        if let Expr::Nested(inner) = expr {
            // The post-order visitor has already canonicalized the nested operand.
            let inner = std::mem::replace(inner, Box::new(Expr::Value(Value::Null.into())));
            *expr = *inner;
            return ControlFlow::Continue(());
        }
        match expr {
            Expr::Identifier(ident) => {
                ident.value = crate::idents::ident_key(ident);
                ident.quote_style = ident
                    .value
                    .bytes()
                    .any(|byte| byte.is_ascii_uppercase())
                    .then_some('"');
            }
            Expr::Function(function) => {
                for part in &mut function.name.0 {
                    match part {
                        ObjectNamePart::Identifier(ident) if ident.quote_style.is_none() => {
                            ident.value.make_ascii_lowercase();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        ControlFlow::Continue(())
    });
    normalized
}
