use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::{FxHashMap, FxHashSet};

pub(in crate::codebase::postgres::query_annotation) fn collect_steps_names<'a>(
    steps: &'a [Step],
    imports: &FxHashMap<String, usize>,
    names: &mut FxHashSet<&'a str>,
) {
    if names.len() == imports.len() {
        return;
    }
    for step in steps {
        match step {
            Step::Hoisted(_, expr)
            | Step::Bind(_, expr)
            | Step::Append(_, expr)
            | Step::Effect(expr)
            | Step::Return(expr) => collect_expr_names(expr, imports, names),
            Step::Branch(test, left, right) => {
                collect_expr_names(test, imports, names);
                collect_steps_names(left, imports, names);
                collect_steps_names(right, imports, names);
            }
            Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => {}
        }
    }
}

pub(in crate::codebase::postgres::query_annotation) fn collect_expr_names<'a>(
    expr: &'a Expr,
    imports: &crate::fx::FxHashMap<String, usize>,
    names: &mut FxHashSet<&'a str>,
) {
    if names.len() == imports.len() {
        return;
    }
    match expr {
        Expr::Name(name) => {
            if imports.contains_key(name) {
                names.insert(name.as_str());
            }
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => collect_expr_names(inner, imports, names),
        Expr::ComputedMember(a, b) | Expr::Append(a, b) | Expr::Binary(_, a, b) => {
            collect_expr_names(a, imports, names);
            collect_expr_names(b, imports, names);
        }
        Expr::Call { callee, args, .. } => {
            collect_expr_names(callee, imports, names);
            for arg in args {
                collect_expr_names(arg, imports, names);
            }
        }
        Expr::Tagged(tag, parts, effects) => {
            if imports.contains_key(tag) {
                names.insert(tag.as_str());
            }
            for part in parts {
                collect_expr_names(part, imports, names);
            }
            for effect in effects {
                collect_expr_names(effect, imports, names);
            }
        }
        Expr::Template(items)
        | Expr::Children(items)
        | Expr::Container(items)
        | Expr::Array(items)
        | Expr::Object(items)
        | Expr::Record(items, _)
        | Expr::Sequence(items)
        | Expr::Alternatives(items)
        | Expr::Opaque(items)
        | Expr::OpaqueWrite {
            children: items, ..
        }
        | Expr::Delete(items, _) => {
            for item in items {
                collect_expr_names(item, imports, names);
            }
        }
        Expr::OptionalCall { call, children } => {
            collect_expr_names(call, imports, names);
            for child in children {
                collect_expr_names(child, imports, names);
            }
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => {
            collect_expr_names(receiver, imports, names);
            collect_expr_names(value, imports, names);
        }
        Expr::Flow(steps) => collect_steps_names(steps, imports, names),
        Expr::Function(function) => collect_steps_names(&function.body, imports, names),
        Expr::Number(_)
        | Expr::Boolean(_)
        | Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Text(_)
        | Expr::ModuleLoad(_) => {}
    }
}
