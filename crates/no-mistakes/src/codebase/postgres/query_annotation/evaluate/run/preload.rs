use super::super::Evaluator;
use super::snapshot::RunSnapshot;
use crate::codebase::postgres::query_annotation::module_ids::ModuleId;
use crate::codebase::postgres::query_annotation::{Expr, Function, Step};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn preload_referenced_imports<F: Fn(&str, &Path) -> Option<PathBuf>>(
    evaluator: &mut Evaluator<'_, F>,
    path: &Path,
    unmodeled: &[&Expr],
    functions: &[Arc<Function>],
    snapshot: &mut RunSnapshot,
    unpromotable: &mut FxHashSet<ModuleId>,
) {
    if evaluator.modules.sources.is_tracking() {
        return;
    }
    let file = &evaluator.files[path];
    let imports = &file.module.imports;
    if imports.is_empty() {
        return;
    }
    let mut names = FxHashSet::default();
    for call in unmodeled {
        collect_expr_names(call, imports, &mut names);
    }
    for function in functions {
        collect_steps_names(&function.body, imports, &mut names);
    }
    if names.is_empty() {
        return;
    }
    let mut targets = names
        .into_iter()
        .filter_map(|name| {
            let index = imports.get(name)?;
            let binding = &file.module.ts.imported_bindings[*index];
            let target = file
                .module
                .target(&binding.specifier, path, &evaluator.resolve)?;
            evaluator
                .files
                .contains_key(target)
                .then(|| target.to_path_buf())
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    for target in targets {
        snapshot.try_promote_module(evaluator, &target, unpromotable);
    }
}

fn collect_steps_names<'a>(
    steps: &'a [Step],
    imports: &crate::fx::FxHashMap<String, usize>,
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

fn collect_expr_names<'a>(
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
