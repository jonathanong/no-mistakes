//! Composite path expressions: templates, `new URL`, and path-helper calls.
use super::super::PathValueKind;
use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::ast::*;

impl<'a> Projector<'a, '_> {
    pub(super) fn template(&mut self, template: &TemplateLiteral<'a>, depth: usize) -> PathOrigin {
        let mut acc: Option<PathOrigin> = None;
        for (index, quasi) in template.quasis.iter().enumerate() {
            let text = crate::ast::template_quasi_text(quasi);
            let pieces = (!text.is_empty())
                .then(|| PathOrigin::literal(&text))
                .into_iter();
            let expression = template
                .expressions
                .get(index)
                .map(|e| self.origin(e, depth + 1));
            for piece in pieces.chain(expression) {
                acc = Some(match acc.take() {
                    None => piece,
                    Some(prefix) => prefix.append(piece),
                });
            }
        }
        acc.unwrap_or_else(|| PathOrigin::literal(""))
            .with_value_kind(PathValueKind::Scalar)
    }

    /// `new URL(relative, base)`.
    pub(super) fn url(&mut self, url: &NewExpression<'a>, depth: usize) -> PathOrigin {
        let is_url = matches!(&url.callee, Expression::Identifier(id) if id.name == "URL");
        let mut args = url.arguments.iter().filter_map(Argument::as_expression);
        let (Some(relative), Some(base), true) = (args.next(), args.next(), is_url) else {
            return PathOrigin::unknown();
        };
        let relative = self.origin(relative, depth + 1);
        self.origin(base, depth + 1)
            .append(relative)
            .with_value_kind(PathValueKind::Mutable)
    }

    pub(super) fn call(&mut self, call: &CallExpression<'a>, depth: usize) -> PathOrigin {
        let callee = call.callee.get_inner_expression();
        if self.node_tmpdir(callee) {
            return PathOrigin::tmpdir(cfg!(windows));
        }
        let (name, receiver) = match callee {
            Expression::Identifier(id) => (id.name.as_str(), None),
            Expression::StaticMemberExpression(member) => {
                (member.property.name.as_str(), callee_root(&member.object))
            }
            _ => return PathOrigin::unknown(),
        };
        if self.is_temp_factory(callee) {
            return PathOrigin::anchored(PathAnchor::Temp).with_value_kind(
                if call.arguments.len() == 1 {
                    PathValueKind::Scalar
                } else {
                    PathValueKind::Mutable
                },
            );
        }
        let mut args = call.arguments.iter().map(Argument::as_expression);
        match name {
            "tmpdir" => PathOrigin::anchored(PathAnchor::SharedTemp),
            "cwd" if receiver == Some("process") => PathOrigin::anchored(PathAnchor::Cwd),
            "join" | "resolve" | "normalize"
                if receiver.is_none_or(|r| matches!(r, "path" | "posix" | "win32")) =>
            {
                let mut acc: Option<PathOrigin> = None;
                for arg in args {
                    let part = arg.map_or_else(PathOrigin::unknown, |e| self.origin(e, depth + 1));
                    acc = Some(acc.take().map_or(part.clone(), |prefix| {
                        if name == "resolve" {
                            prefix.resolve_append(part)
                        } else {
                            prefix.append(part)
                        }
                    }));
                }
                acc.unwrap_or_else(PathOrigin::unknown)
                    .with_value_kind(PathValueKind::Scalar)
            }
            "dirname" | "fileURLToPath" | "pathToFileURL" | "realpath" | "realpathSync" => {
                let Some(Some(inner)) = args.next() else {
                    return PathOrigin::unknown();
                };
                let origin = self.origin(inner, depth + 1);
                let kind = if name == "pathToFileURL"
                    || (matches!(name, "realpath" | "realpathSync") && call.arguments.len() != 1)
                {
                    PathValueKind::Mutable
                } else {
                    PathValueKind::Scalar
                };
                if name == "dirname" {
                    origin.parent().with_value_kind(kind)
                } else {
                    origin.with_value_kind(kind)
                }
            }
            _ => self
                .getter_origin(call, depth)
                .or_else(|| self.imported_getter(call))
                .unwrap_or_else(PathOrigin::unknown),
        }
    }
}

pub(in super::super) fn imported_locals(
    facts: &super::super::super::TypeScriptModuleFacts,
) -> crate::fx::FxHashSet<&str> {
    let mut locals: crate::fx::FxHashSet<&str> = facts
        .imports
        .iter()
        .filter(|import| !import.type_only)
        .flat_map(|import| import.bindings.iter())
        .filter(|binding| !binding.type_only)
        .map(|binding| binding.local.as_str())
        .collect();
    locals.extend(
        facts
            .require_bindings
            .iter()
            .map(|binding| binding.local.as_str()),
    );
    locals
}

/// Leftmost identifier of a static member chain.
pub(in super::super) fn callee_root<'a>(expression: &'a Expression<'_>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => callee_root(&member.object),
        _ => None,
    }
}

pub(super) fn disposable_path(call: &CallExpression<'_>, key: &str) -> Option<PathOrigin> {
    let disposable = matches!(call.callee.get_inner_expression(),
        Expression::Identifier(id) if id.name == "mkdtempDisposableSync")
        || matches!(call.callee.get_inner_expression(),
        Expression::StaticMemberExpression(member) if member.property.name == "mkdtempDisposableSync");
    if key != "path" || !disposable {
        return None;
    }
    let kind = if call.arguments.len() == 1 {
        PathValueKind::Scalar
    } else {
        PathValueKind::Mutable
    };
    Some(PathOrigin::anchored(PathAnchor::Temp).with_value_kind(kind))
}
