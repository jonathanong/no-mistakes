use super::super::Function;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::FxHashSet;

#[derive(Default)]
pub(super) struct Reads {
    pub values: FxHashSet<String>,
    pub identities: FxHashSet<String>,
}
pub(super) fn names(function: &Function) -> Reads {
    let mut names = Reads::default();
    for step in &function.body {
        match step {
            Step::Bind(_, value)
            | Step::Hoisted(_, value)
            | Step::Effect(value)
            | Step::Return(value) => expression(value, &mut names),
            Step::Append(name, value) => {
                names.values.insert(name.clone());
                expression(value, &mut names);
            }
            _ => {}
        }
    }
    let shadowed = super::super::calls::scopes::shadow_names(function);
    for set in [&mut names.values, &mut names.identities] {
        set.retain(|name| !shadowed.contains(name));
    }
    names.identities.retain(|name| !names.values.contains(name));
    names
}
fn expression(expr: &Expr, names: &mut Reads) {
    match expr {
        Expr::Name(name) => {
            names.values.insert(name.clone());
        }
        Expr::Await(value)
        | Expr::Spread(value)
        | Expr::Index(value, _)
        | Expr::Member(value, _)
        | Expr::OpaqueCallback(value)
        | Expr::Discard(value) => expression(value, names),
        Expr::Call { callee, args, .. } => {
            expression(callee, names);
            for value in args {
                expression(value, names);
            }
        }
        Expr::Template(values)
        | Expr::Children(values)
        | Expr::Sequence(values)
        | Expr::Alternatives(values)
        | Expr::Opaque(values)
        | Expr::Delete(values, _)
        | Expr::OpaqueWrite {
            children: values, ..
        } => {
            for value in values {
                expression(value, names);
            }
        }
        Expr::Append(left, right) => {
            expression(left, names);
            expression(right, names);
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => {
            if let Expr::Name(name) = receiver.as_ref() {
                names.identities.insert(name.clone());
            } else {
                expression(receiver, names);
            }
            expression(value, names);
        }
        Expr::Tagged(name, substitutions, values) => {
            names.values.insert(name.clone());
            for value in substitutions.iter().chain(values) {
                expression(value, names);
            }
        }
        _ => {}
    }
}
