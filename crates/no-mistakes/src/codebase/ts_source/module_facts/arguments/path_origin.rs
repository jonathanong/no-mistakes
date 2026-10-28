//! Static provenance of the first path argument of a call through an import.
//!
//! Syntactic heuristics only: `join`/`resolve`, `tmpdir`, `mkdtemp`, `cwd`, and
//! `import.meta.dirname` are recognised by spelling, and only immutable `const`
//! aliases are followed. Everything else is `Unknown`, never guessed.
mod composite;

use self::composite::callee_root;
pub(super) use self::composite::imported_locals;
use super::{PathAnchor, PathOrigin, Projector};
use crate::fx::FxHashSet;
use oxc_ast::{ast::*, AstKind};

impl PathOrigin {
    pub(crate) fn unknown() -> Self {
        Self {
            anchor: PathAnchor::Unknown,
            segments: Vec::new(),
            dynamic: true,
        }
    }

    fn anchored(anchor: PathAnchor) -> Self {
        Self {
            anchor,
            segments: Vec::new(),
            dynamic: false,
        }
    }

    fn literal(text: &str) -> Self {
        let absolute = text.starts_with(['/', '\\']) || text.as_bytes().get(1) == Some(&b':');
        let mut origin = Self {
            anchor: if absolute {
                PathAnchor::Absolute
            } else {
                PathAnchor::Relative
            },
            segments: Vec::new(),
            dynamic: false,
        };
        origin.extend(
            text.split(['/', '\\'])
                .filter(|part| !part.is_empty() && *part != ".")
                .take(32),
        );
        origin
    }

    /// Add components, resolving `..` so `fixtures/../src` is not a fixture path.
    fn extend<'s>(&mut self, parts: impl IntoIterator<Item = &'s str>) {
        for part in parts {
            if part == ".." && self.segments.last().is_some_and(|last| last != "..") {
                self.segments.pop();
            } else {
                self.segments.push(part.to_owned());
            }
        }
    }

    /// Append `tail` as trailing components of `self`.
    fn append(mut self, tail: Self) -> Self {
        self.dynamic |=
            tail.dynamic || !matches!(tail.anchor, PathAnchor::Relative | PathAnchor::Absolute);
        self.extend(tail.segments.iter().map(String::as_str));
        self
    }
}

impl<'a> Projector<'a, '_> {
    pub(super) fn call_path(
        &mut self,
        call: &CallExpression<'a>,
        imported: &FxHashSet<&str>,
    ) -> Option<PathOrigin> {
        if !imported.contains(callee_root(&call.callee)?) {
            return None;
        }
        let first = call.arguments.first()?;
        let Some(first) = first.as_expression() else {
            return Some(PathOrigin::unknown());
        };
        let origin = self.origin(first, 0);
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

    fn origin(&mut self, expression: &Expression<'a>, depth: usize) -> PathOrigin {
        if depth >= 16 || !self.take() {
            return PathOrigin::unknown();
        }
        match expression.get_inner_expression() {
            Expression::StringLiteral(text) => PathOrigin::literal(&crate::ast::js_literal(text)),
            Expression::TemplateLiteral(template) => self.template(template, depth),
            Expression::Identifier(id) => self.identifier(id, depth),
            Expression::AwaitExpression(inner) => self.origin(&inner.argument, depth + 1),
            Expression::StaticMemberExpression(member) => {
                let module = matches!(&member.object, Expression::ImportMeta(_))
                    && matches!(
                        member.property.name.as_str(),
                        "dirname" | "filename" | "url"
                    );
                if module {
                    PathOrigin::anchored(PathAnchor::Module)
                } else {
                    PathOrigin::unknown()
                }
            }
            Expression::NewExpression(url) => self.url(url, depth),
            Expression::CallExpression(call) => self.call(call, depth),
            _ => PathOrigin::unknown(),
        }
    }

    fn identifier(&mut self, id: &IdentifierReference<'a>, depth: usize) -> PathOrigin {
        let global = matches!(id.name.as_str(), "__dirname" | "__filename");
        if global && self.binding_symbol(id).is_none() {
            return PathOrigin::anchored(PathAnchor::Module);
        }
        match self.const_init(id) {
            Some(init) => self.origin(init, depth + 1),
            None => PathOrigin::unknown(),
        }
    }

    fn binding_symbol(&self, id: &IdentifierReference<'a>) -> Option<oxc_semantic::SymbolId> {
        let reference = id.reference_id.get()?;
        self.semantic.scoping().get_reference(reference).symbol_id()
    }

    /// Initializer of an immutable `const` that `id` reads after its declaration.
    fn const_init(&mut self, id: &IdentifierReference<'a>) -> Option<&'a Expression<'a>> {
        let semantic = self.semantic;
        let symbol = self.binding_symbol(id)?;
        if !self.immutable_binding(symbol) {
            return None;
        }
        let AstKind::VariableDeclarator(declaration) = semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        else {
            return None;
        };
        let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
            return None;
        };
        let init = declaration.init.as_ref()?;
        (binding.symbol_id.get() == Some(symbol) && id.span.start >= declaration.span.end)
            .then_some(init)
    }
}
