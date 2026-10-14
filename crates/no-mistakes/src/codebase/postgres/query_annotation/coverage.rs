//! Retain uncertainty for helper invocations omitted by the supported summaries.
use super::{Expr, Step};
use crate::fx::{fx_set, FxHashSet};
mod calls;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;

pub(super) fn collect(
    program: &Program<'_>,
    source: &str,
    roots: &[Step],
) -> (Vec<Expr>, FxHashSet<(u32, u32)>) {
    fn expr(
        value: &Expr,
        covered: &mut FxHashSet<(u32, u32)>,
        branches: &mut FxHashSet<(u32, u32)>,
        inside: bool,
    ) {
        match value {
            Expr::Call {
                callee,
                args,
                start,
                end,
            } => {
                if inside {
                    branches.insert((*start, *end));
                } else {
                    covered.insert((*start, *end));
                }
                expr(callee, covered, branches, inside);
                for arg in args {
                    expr(arg, covered, branches, inside);
                }
            }
            Expr::Function(function) => steps(&function.body, covered, branches, inside),
            Expr::Flow(body) => steps(body, covered, branches, inside),
            Expr::Template(parts)
            | Expr::Children(parts)
            | Expr::Array(parts)
            | Expr::Container(parts)
            | Expr::Object(parts)
            | Expr::Record(parts, _)
            | Expr::Sequence(parts)
            | Expr::Opaque(parts)
            | Expr::OpaqueWrite {
                children: parts, ..
            }
            | Expr::Delete(parts, _)
            | Expr::Alternatives(parts) => {
                for part in parts {
                    expr(part, covered, branches, inside);
                }
            }
            Expr::Tagged(_, parts, effects) => {
                for part in parts.iter().chain(effects) {
                    expr(part, covered, branches, inside);
                }
            }
            Expr::Await(value)
            | Expr::Spread(value)
            | Expr::OpaqueCallback(value)
            | Expr::Discard(value)
            | Expr::Member(value, _)
            | Expr::Index(value, _) => expr(value, covered, branches, inside),
            Expr::Append(base, tail)
            | Expr::Binary(_, base, tail)
            | Expr::ComputedMember(base, tail) => {
                expr(base, covered, branches, inside);
                expr(tail, covered, branches, inside);
            }
            Expr::SlotWrite {
                receiver, value, ..
            } => {
                expr(receiver, covered, branches, inside);
                expr(value, covered, branches, inside);
            }
            _ => {}
        }
    }
    fn steps(
        values: &[Step],
        covered: &mut FxHashSet<(u32, u32)>,
        branches: &mut FxHashSet<(u32, u32)>,
        inside: bool,
    ) {
        for value in values {
            match value {
                Step::Bind(_, value)
                | Step::Hoisted(_, value)
                | Step::Append(_, value)
                | Step::Effect(value)
                | Step::Return(value) => expr(value, covered, branches, inside),
                Step::Branch(test, left, right) => {
                    expr(test, covered, branches, true);
                    steps(left, covered, branches, true);
                    steps(right, covered, branches, true);
                }
                _ => {}
            }
        }
    }
    let mut covered = fx_set();
    let mut branches = fx_set();
    steps(roots, &mut covered, &mut branches, false);
    let mut calls = calls::Calls {
        source,
        covered,
        values: Vec::new(),
        scopes: Vec::new(),
    };
    calls.visit_program(program);
    (calls.values, branches)
}
