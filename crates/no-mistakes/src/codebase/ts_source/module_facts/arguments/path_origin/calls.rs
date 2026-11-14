//! Path arguments for imported calls and the exact import.meta glob API.
use super::composite::callee_root;
use super::{PathAnchor, PathOrigin, Projector};
use crate::fx::FxHashSet;
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    fn imported_callee(
        &mut self,
        callee: &Expression<'a>,
        imported: &FxHashSet<&str>,
        depth: usize,
    ) -> bool {
        if depth >= 16 || !self.take() {
            return false;
        }
        let Some(root) = callee_root(callee) else {
            return false;
        };
        if imported.contains(root) {
            return true;
        }
        let Expression::Identifier(id) = callee.get_inner_expression() else {
            return false;
        };
        self.const_init(id)
            .is_some_and(|init| self.imported_callee(init, imported, depth + 1))
    }

    pub(in super::super) fn call_path(
        &mut self,
        call: &CallExpression<'a>,
        imported: &FxHashSet<&str>,
    ) -> Option<PathOrigin> {
        let meta_glob = is_import_meta_glob(&call.callee);
        if !meta_glob && !self.imported_callee(&call.callee, imported, 0) {
            return None;
        }
        let mut origin = self.argument_origin(call, 0)?;
        if meta_glob {
            // Vite patterns are relative to the module, not the process cwd.
            if origin.anchor == PathAnchor::Relative {
                origin.anchor = PathAnchor::Module;
            }
            return Some(origin);
        }
        // `glob(pattern, { cwd })` resolves the pattern against `cwd`; options this
        // pass cannot read leave a relative pattern with an unknown base.
        if origin.anchor != PathAnchor::Relative {
            return Some(origin);
        }
        let Some(options) = call.arguments.get(1).and_then(Argument::as_expression) else {
            return Some(origin);
        };
        let kind = match options.get_inner_expression() {
            Expression::Identifier(id) => self
                .const_init(id)
                .map(Expression::get_inner_expression)
                .unwrap_or(options.get_inner_expression()),
            other => other,
        };
        if !matches!(
            kind,
            Expression::ObjectExpression(_) | Expression::Identifier(_)
        ) {
            return Some(origin);
        }
        let (mut paths, unknown) = self.object_shape(options, 0);
        Some(match paths.remove("cwd") {
            Some(cwd) => cwd.append(origin),
            None if unknown => PathOrigin::unknown(),
            None => origin,
        })
    }
}

fn is_import_meta_glob(expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.object.get_inner_expression(),
                Expression::ImportMeta(_)
            ) && matches!(member.property.name.as_str(), "glob" | "globEager")
        }
        Expression::ComputedMemberExpression(member) => {
            matches!(
                member.object.get_inner_expression(),
                Expression::ImportMeta(_)
            ) && matches!(member.expression.get_inner_expression(),
                    Expression::StringLiteral(name) if matches!(name.value.as_str(), Some("glob" | "globEager")))
        }
        _ => false,
    }
}
