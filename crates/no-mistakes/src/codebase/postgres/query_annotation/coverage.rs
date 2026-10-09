//! Retain uncertainty for helper invocations omitted by the supported summaries.
use super::{Expr, Step};
use crate::fx::{fx_set, FxHashSet};
mod calls;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

pub(super) fn collect(program: &Program<'_>, source: &str, roots: &[Step]) -> Vec<Expr> {
    fn expr(value: &Expr, covered: &mut FxHashSet<u32>) {
        match value {
            Expr::Call {
                callee,
                args,
                start,
            } => {
                covered.insert(*start);
                expr(callee, covered);
                for arg in args {
                    expr(arg, covered);
                }
            }
            Expr::Function(function) => steps(&function.body, covered),
            Expr::Template(parts)
            | Expr::Children(parts)
            | Expr::Opaque(parts)
            | Expr::Delete(parts)
            | Expr::Alternatives(parts) => {
                for part in parts {
                    expr(part, covered);
                }
            }
            Expr::Tagged(_, parts, effects) => {
                for part in parts.iter().chain(effects) {
                    expr(part, covered);
                }
            }
            Expr::Await(value)
            | Expr::Spread(value)
            | Expr::OpaqueCallback(value)
            | Expr::Index(value, _) => expr(value, covered),
            Expr::Append(base, tail) => {
                expr(base, covered);
                expr(tail, covered);
            }
            _ => {}
        }
    }
    fn steps(values: &[Step], covered: &mut FxHashSet<u32>) {
        for value in values {
            match value {
                Step::Bind(_, value)
                | Step::Hoisted(_, value)
                | Step::Append(_, value)
                | Step::Effect(value)
                | Step::Return(value) => expr(value, covered),
                _ => {}
            }
        }
    }
    let mut covered = fx_set();
    steps(roots, &mut covered);
    let mut calls = calls::Calls {
        source,
        covered,
        values: Vec::new(),
        scopes: Vec::new(),
    };
    calls.visit_program(program);
    calls.values
}
