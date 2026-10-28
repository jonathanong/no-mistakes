//! Composite path expressions: templates, `new URL`, and path-helper calls.
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
    }

    /// `new URL(relative, base)`.
    pub(super) fn url(&mut self, url: &NewExpression<'a>, depth: usize) -> PathOrigin {
        let is_url = matches!(&url.callee, Expression::Identifier(id) if id.name == "URL");
        let mut args = url.arguments.iter().filter_map(Argument::as_expression);
        let (Some(relative), Some(base), true) = (args.next(), args.next(), is_url) else {
            return PathOrigin::unknown();
        };
        let relative = self.origin(relative, depth + 1);
        self.origin(base, depth + 1).append(relative)
    }

    pub(super) fn call(&mut self, call: &CallExpression<'a>, depth: usize) -> PathOrigin {
        let callee = call.callee.get_inner_expression();
        let (name, receiver) = match callee {
            Expression::Identifier(id) => (id.name.as_str(), None),
            Expression::StaticMemberExpression(member) => {
                (member.property.name.as_str(), callee_root(&member.object))
            }
            _ => return PathOrigin::unknown(),
        };
        let mut args = call.arguments.iter().map(Argument::as_expression);
        match name {
            "tmpdir" | "mkdtemp" | "mkdtempSync" => PathOrigin::anchored(PathAnchor::Temp),
            "cwd" if receiver == Some("process") => PathOrigin::anchored(PathAnchor::Cwd),
            "join" | "resolve" | "normalize"
                if receiver.is_none_or(|r| matches!(r, "path" | "posix" | "win32")) =>
            {
                let mut acc: Option<PathOrigin> = None;
                for arg in args {
                    let part = arg.map_or_else(PathOrigin::unknown, |e| self.origin(e, depth + 1));
                    acc = Some(
                        acc.take()
                            .map_or(part.clone(), |prefix| prefix.append(part)),
                    );
                }
                acc.unwrap_or_else(PathOrigin::unknown)
            }
            "dirname" | "fileURLToPath" | "pathToFileURL" | "realpath" | "realpathSync" => {
                let Some(Some(inner)) = args.next() else {
                    return PathOrigin::unknown();
                };
                let mut origin = self.origin(inner, depth + 1);
                if name == "dirname" {
                    origin.segments.pop();
                }
                origin
            }
            _ => PathOrigin::unknown(),
        }
    }
}

/// Leftmost identifier of a static member chain.
pub(super) fn callee_root<'a>(expression: &'a Expression<'_>) -> Option<&'a str> {
    match expression.get_inner_expression() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => callee_root(&member.object),
        _ => None,
    }
}
