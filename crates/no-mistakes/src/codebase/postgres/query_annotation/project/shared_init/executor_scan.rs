use super::File;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::postgres::query_annotation::{Expr, Step};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

pub(in crate::codebase::postgres::query_annotation) fn is_statically_pure_top_level(
    file: &File<'_>,
    path: &Path,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> bool {
    if file.facts.precise_records && has_unresolved_relative_import(file, path, resolve) {
        return false;
    }
    file.facts
        .roots
        .iter()
        .all(|step| is_pure_top_level_step(step, file))
}

fn has_unresolved_relative_import(
    file: &File<'_>,
    path: &Path,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> bool {
    file.module.ts.imports.iter().any(|import| {
        import.kind == ImportKind::Static
            && import.function_scope.is_none()
            && import.specifier.starts_with('.')
            && file
                .module
                .target(&import.specifier, path, resolve)
                .is_none()
    })
}

fn is_pure_top_level_step(step: &Step, file: &File<'_>) -> bool {
    match step {
        Step::Reserve(_) | Step::Var(_) => true,
        Step::Hoisted(_, expr) | Step::Effect(expr) => is_pure_top_level_expr(expr, file),
        Step::Bind(name, expr) => {
            !file.module.imports.contains_key(name) && is_pure_top_level_expr(expr, file)
        }
        Step::Branch(..)
        | Step::Append(..)
        | Step::Return(_)
        | Step::Unsupported
        | Step::PotentialCalls(_) => false,
    }
}

fn is_pure_top_level_expr(expr: &Expr, file: &File<'_>) -> bool {
    match expr {
        Expr::Text(_)
        | Expr::Number(_)
        | Expr::Boolean(_)
        | Expr::Primitive
        | Expr::Unknown
        | Expr::Unsupported
        | Expr::Function(_) => true,
        Expr::Name(name) => name != "arguments",
        Expr::Await(inner)
        | Expr::Spread(inner)
        | Expr::Index(inner, _)
        | Expr::Member(inner, _)
        | Expr::Discard(inner) => is_pure_top_level_expr(inner, file),
        Expr::ComputedMember(a, b) | Expr::Binary(_, a, b) => {
            is_pure_top_level_expr(a, file) && is_pure_top_level_expr(b, file)
        }
        Expr::Template(items)
        | Expr::Children(items)
        | Expr::Container(items)
        | Expr::Object(items)
        | Expr::Record(items, _)
        | Expr::Sequence(items)
        | Expr::Alternatives(items)
        | Expr::Opaque(items) => items.iter().all(|item| is_pure_top_level_expr(item, file)),
        Expr::Call {
            callee,
            args,
            start,
            end,
        } => {
            !file.executors.contains(&(*start, *end))
                && args.iter().all(|arg| is_pure_scalar_expr(arg, None))
                && is_pure_local_scalar_callee(callee, file)
        }
        Expr::Array(_)
        | Expr::Tagged(..)
        | Expr::SlotWrite { .. }
        | Expr::OpaqueWrite { .. }
        | Expr::Delete(..)
        | Expr::ModuleLoad(_)
        | Expr::OptionalCall { .. }
        | Expr::OpaqueCallback(_)
        | Expr::Append(..)
        | Expr::Flow(_) => false,
    }
}

fn is_pure_local_scalar_callee(callee: &Expr, file: &File<'_>) -> bool {
    let Expr::Name(name) = callee else {
        return false;
    };
    if file.module.imports.contains_key(name) {
        return false;
    }
    let Some(Expr::Function(function)) = file.facts.globals.get(name) else {
        return false;
    };
    if function.asynchronous {
        return false;
    }
    let mut locals: FxHashSet<&str> = function.params.iter().map(String::as_str).collect();
    if let Some(self_name) = &function.self_name {
        locals.insert(self_name.as_str());
    }
    for step in &function.body {
        match step {
            Step::Reserve(names) => locals.extend(names.iter().map(String::as_str)),
            Step::Var(var) | Step::Hoisted(var, _) => {
                locals.insert(var.as_str());
            }
            _ => {}
        }
    }
    function.body.iter().all(|step| match step {
        Step::Reserve(_) | Step::Var(_) => true,
        Step::Hoisted(_, expr) | Step::Return(expr) => is_pure_scalar_expr(expr, Some(&locals)),
        Step::Bind(lhs, expr) => {
            locals.contains(lhs.as_str()) && is_pure_scalar_expr(expr, Some(&locals))
        }
        Step::Effect(_)
        | Step::Branch(..)
        | Step::Append(..)
        | Step::Unsupported
        | Step::PotentialCalls(_) => false,
    })
}

fn is_pure_scalar_expr(expr: &Expr, locals: Option<&FxHashSet<&str>>) -> bool {
    match expr {
        Expr::Text(_) | Expr::Number(_) | Expr::Boolean(_) | Expr::Primitive | Expr::Unknown => {
            true
        }
        Expr::Name(name) => {
            name != "arguments" && locals.is_none_or(|set| set.contains(name.as_str()))
        }
        Expr::Binary(_, a, b) => is_pure_scalar_expr(a, locals) && is_pure_scalar_expr(b, locals),
        Expr::Template(items) => items.iter().all(|item| is_pure_scalar_expr(item, locals)),
        _ => false,
    }
}
