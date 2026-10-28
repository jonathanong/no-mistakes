//! Path arguments for imported calls and the exact import.meta glob API.
use super::composite::callee_root;
use super::{PathAnchor, PathOrigin, Projector};
use crate::fx::FxHashSet;
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(in super::super) fn call_path(
        &mut self,
        call: &CallExpression<'a>,
        imported: &FxHashSet<&str>,
    ) -> Option<PathOrigin> {
        let meta_glob = is_import_meta_glob(&call.callee);
        if !meta_glob && !callee_root(&call.callee).is_some_and(|root| imported.contains(root)) {
            return None;
        }
        let first = call.arguments.first()?;
        let Some(first) = first.as_expression() else {
            return Some(PathOrigin::unknown());
        };
        let mut origin = self.origin(first, 0);
        if meta_glob {
            // Vite patterns are relative to the module, not the process cwd.
            if origin.anchor == PathAnchor::Relative {
                origin.anchor = PathAnchor::Module;
            }
            return Some(origin);
        }
        // `glob(pattern, { cwd })` resolves the pattern against `cwd`; options this
        // pass cannot read leave a relative pattern with an unknown base.
        let options = call
            .arguments
            .get(1)
            .and_then(Argument::as_expression)
            .map(Expression::get_inner_expression);
        let options = match options {
            Some(Expression::Identifier(id)) => self
                .const_init(id)
                .map(Expression::get_inner_expression)
                .or(options),
            other => other,
        };
        if origin.anchor != PathAnchor::Relative {
            return Some(origin);
        }
        Some(match options {
            None => origin,
            Some(Expression::ObjectExpression(options)) => {
                let cwd = options.properties.iter().find_map(|p| match p {
                    ObjectPropertyKind::ObjectProperty(p)
                        if !p.computed
                            && crate::ast::property_key_name(&p.key).as_deref() == Some("cwd") =>
                    {
                        Some(&p.value)
                    }
                    _ => None,
                });
                let spread = options
                    .properties
                    .iter()
                    .any(|p| matches!(p, ObjectPropertyKind::SpreadProperty(_)));
                match cwd {
                    Some(cwd) => self.origin(cwd, 0).append(origin),
                    None if spread => PathOrigin::unknown(),
                    None => origin,
                }
            }
            Some(Expression::Identifier(_)) => PathOrigin::unknown(),
            Some(_) => origin,
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
