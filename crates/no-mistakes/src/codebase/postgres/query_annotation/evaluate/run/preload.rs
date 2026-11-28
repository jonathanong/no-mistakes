use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::{FxHashMap, FxHashSet};

pub(in crate::codebase::postgres::query_annotation) fn collect_steps_names<'a>(
    steps: &'a [Step],
    imports: &FxHashMap<String, usize>,
    names: &mut FxHashSet<&'a str>,
) {
    collect_steps_with_shadowed(steps, imports, &FxHashSet::default(), true, names);
}

pub(in crate::codebase::postgres::query_annotation) fn collect_top_level_steps_names<'a>(
    steps: &'a [Step],
    imports: &FxHashMap<String, usize>,
    names: &mut FxHashSet<&'a str>,
) {
    collect_steps_with_shadowed(steps, imports, &FxHashSet::default(), false, names);
}

pub(in crate::codebase::postgres::query_annotation) fn collect_expr_names<'a>(
    expr: &'a Expr,
    imports: &FxHashMap<String, usize>,
    names: &mut FxHashSet<&'a str>,
) {
    collect_expr_with_shadowed(expr, imports, &FxHashSet::default(), true, names);
}

fn collect_steps_with_shadowed<'a>(
    steps: &'a [Step],
    imports: &FxHashMap<String, usize>,
    shadowed: &FxHashSet<&'a str>,
    descend_functions: bool,
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
            | Step::Return(expr) => {
                collect_expr_with_shadowed(expr, imports, shadowed, descend_functions, names)
            }
            Step::Branch(test, left, right) => {
                collect_expr_with_shadowed(test, imports, shadowed, descend_functions, names);
                collect_steps_with_shadowed(left, imports, shadowed, descend_functions, names);
                collect_steps_with_shadowed(right, imports, shadowed, descend_functions, names);
            }
            Step::Reserve(_) | Step::Var(_) | Step::Unsupported | Step::PotentialCalls(_) => {}
        }
    }
}

fn collect_declared_names<'a>(steps: &'a [Step], out: &mut FxHashSet<&'a str>) {
    for step in steps {
        match step {
            Step::Reserve(names) => {
                for name in names {
                    out.insert(name.as_str());
                }
            }
            Step::Var(name) | Step::Hoisted(name, _) | Step::Bind(name, _) => {
                out.insert(name.as_str());
            }
            Step::Branch(_, left, right) => {
                collect_declared_names(left, out);
                collect_declared_names(right, out);
            }
            Step::Append(_, _)
            | Step::Effect(_)
            | Step::Return(_)
            | Step::Unsupported
            | Step::PotentialCalls(_) => {}
        }
    }
}

fn collect_expr_with_shadowed<'a>(
    expr: &'a Expr,
    imports: &FxHashMap<String, usize>,
    shadowed: &FxHashSet<&'a str>,
    descend_functions: bool,
    names: &mut FxHashSet<&'a str>,
) {
    if names.len() == imports.len() {
        return;
    }
    match expr {
        Expr::Name(name) => {
            if imports.contains_key(name) && !shadowed.contains(name.as_str()) {
                names.insert(name.as_str());
            }
        }
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::OpaqueCallback(inner)
        | Expr::Discard(inner) => {
            collect_expr_with_shadowed(inner, imports, shadowed, descend_functions, names)
        }
        Expr::ComputedMember(a, b) | Expr::Append(a, b) | Expr::Binary(_, a, b) => {
            collect_expr_with_shadowed(a, imports, shadowed, descend_functions, names);
            collect_expr_with_shadowed(b, imports, shadowed, descend_functions, names);
        }
        Expr::Call { callee, args, .. } => {
            collect_expr_with_shadowed(callee, imports, shadowed, descend_functions, names);
            for arg in args {
                collect_expr_with_shadowed(arg, imports, shadowed, descend_functions, names);
            }
        }
        Expr::Tagged(tag, parts, effects) => {
            if imports.contains_key(tag) && !shadowed.contains(tag.as_str()) {
                names.insert(tag.as_str());
            }
            for part in parts {
                collect_expr_with_shadowed(part, imports, shadowed, descend_functions, names);
            }
            for effect in effects {
                collect_expr_with_shadowed(effect, imports, shadowed, descend_functions, names);
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
                collect_expr_with_shadowed(item, imports, shadowed, descend_functions, names);
            }
        }
        Expr::OptionalCall { call, children } => {
            collect_expr_with_shadowed(call, imports, shadowed, descend_functions, names);
            for child in children {
                collect_expr_with_shadowed(child, imports, shadowed, descend_functions, names);
            }
        }
        Expr::SlotWrite {
            receiver, value, ..
        } => {
            collect_expr_with_shadowed(receiver, imports, shadowed, descend_functions, names);
            collect_expr_with_shadowed(value, imports, shadowed, descend_functions, names);
        }
        Expr::Flow(steps) => {
            collect_steps_with_shadowed(steps, imports, shadowed, descend_functions, names)
        }
        Expr::Function(function) if descend_functions => {
            let mut inner = shadowed.clone();
            for param in &function.params {
                inner.insert(param.as_str());
            }
            if let Some(self_name) = &function.self_name {
                inner.insert(self_name.as_str());
            }
            collect_declared_names(&function.body, &mut inner);
            collect_steps_with_shadowed(&function.body, imports, &inner, true, names);
        }
        Expr::Function(_)
        | Expr::Number(_)
        | Expr::Boolean(_)
        | Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Text(_)
        | Expr::ModuleLoad(_) => {}
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "preload_tests.rs"]
mod tests;
