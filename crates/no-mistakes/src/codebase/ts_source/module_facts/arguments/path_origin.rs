//! Static provenance of the first path argument of a call through an import.
//!
//! Syntactic heuristics only: `join`/`resolve`, `tmpdir`, `mkdtemp`, `cwd`, and
//! `import.meta.dirname` are recognised by spelling, and only immutable `const`
//! aliases are followed. Everything else is `Unknown`, never guessed.
mod composite;

use self::composite::callee_root;
use super::{PathAnchor, PathOrigin, Projector};
use crate::fx::FxHashSet;
use oxc_ast::{ast::*, AstKind};

pub(super) fn imported_locals(facts: &super::super::TypeScriptModuleFacts) -> FxHashSet<&str> {
    facts
        .imports
        .iter()
        .filter(|import| !import.type_only)
        .flat_map(|import| import.bindings.iter())
        .filter(|binding| !binding.type_only)
        .map(|binding| binding.local.as_str())
        .collect()
}

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
        Self {
            anchor: if absolute {
                PathAnchor::Absolute
            } else {
                PathAnchor::Relative
            },
            segments: text
                .split(['/', '\\'])
                .filter(|part| !part.is_empty() && *part != ".")
                .take(32)
                .map(str::to_owned)
                .collect(),
            dynamic: false,
        }
    }

    /// Append `tail` as trailing components of `self`.
    fn append(mut self, tail: Self) -> Self {
        self.dynamic |=
            tail.dynamic || !matches!(tail.anchor, PathAnchor::Relative | PathAnchor::Absolute);
        self.segments.extend(tail.segments);
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
        // `glob(pattern, { cwd })` resolves the pattern against `cwd`.
        let cwd = call
            .arguments
            .get(1)
            .and_then(Argument::as_expression)
            .and_then(|options| match options.get_inner_expression() {
                Expression::ObjectExpression(options) => options.properties.iter().find_map(|p| {
                    let ObjectPropertyKind::ObjectProperty(p) = p else {
                        return None;
                    };
                    (!p.computed && crate::ast::property_key_name(&p.key).as_deref() == Some("cwd"))
                        .then_some(&p.value)
                }),
                _ => None,
            });
        Some(match cwd {
            Some(cwd) if origin.anchor == PathAnchor::Relative => {
                self.origin(cwd, 0).append(origin)
            }
            _ => origin,
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
        let semantic = self.semantic;
        let Some(reference) = id.reference_id.get() else {
            return PathOrigin::unknown();
        };
        let Some(symbol) = semantic.scoping().get_reference(reference).symbol_id() else {
            let global = matches!(id.name.as_str(), "__dirname" | "__filename");
            return if global {
                PathOrigin::anchored(PathAnchor::Module)
            } else {
                PathOrigin::unknown()
            };
        };
        if !self.immutable_binding(symbol) {
            return PathOrigin::unknown();
        }
        let AstKind::VariableDeclarator(declaration) = semantic
            .nodes()
            .kind(semantic.scoping().symbol_declaration(symbol))
        else {
            return PathOrigin::unknown();
        };
        let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
            return PathOrigin::unknown();
        };
        match &declaration.init {
            Some(init)
                if binding.symbol_id.get() == Some(symbol)
                    && id.span.start >= declaration.span.end =>
            {
                self.origin(init, depth + 1)
            }
            _ => PathOrigin::unknown(),
        }
    }
}
