//! Static provenance of the first path argument of a call through an import.
//!
//! Syntactic heuristics only: `join`/`resolve`, `tmpdir`, `mkdtemp`, `cwd`, and
//! `import.meta.dirname` are recognised by spelling, and only immutable `const`
//! aliases are followed. Everything else is `Unknown`, never guessed.
mod alternatives;
mod bindings;
mod calls;
mod composite;
mod defaults;
mod destructure;
mod getters;
mod forwarded;
mod members;
mod nullish;
mod object_paths;
mod origins;
mod patterns;
mod positions;
mod sources;
mod tmpdir;
pub(super) use self::composite::{callee_root, imported_locals};
pub(super) use self::sources::{env_name, is_process_env, process_global};
use super::{PathAnchor, PathOrigin, Projector};
use oxc_ast::{ast::*, AstKind};

impl<'a> Projector<'a, '_> {
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
                    self.member_origin(&member.object, member.property.name.as_str(), depth)
                        .unwrap_or_else(|| self.env_origin(expression))
                }
            }
            Expression::ComputedMemberExpression(_) => self.env_origin(expression),
            Expression::LogicalExpression(logical)
                if matches!(
                    logical.operator,
                    LogicalOperator::Coalesce | LogicalOperator::Or
                ) =>
            {
                if self.definitely_nullish(&logical.left, depth + 1) {
                    return self.origin(&logical.right, depth + 1);
                }
                let left = self.origin(&logical.left, depth + 1);
                if left.env.is_some() {
                    left
                } else {
                    let right = self.origin(&logical.right, depth + 1);
                    PathOrigin::alternatives(vec![left, right])
                }
            }
            Expression::ChainExpression(chain) => self.chain_origin(chain, depth),
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
            Some(init) => {
                let origin = self.origin(init, depth + 1);
                // A captured env read ran when the alias was initialised, not at the
                // call, so a later owned write does not cover it.
                if origin.atoms().iter().any(|atom| atom.env.is_some() || atom.getter.is_some()) {
                    PathOrigin::unknown()
                } else {
                    origin
                }
            }
            None => {
                let mut origin = self
                    .destructured_origin(id, depth)
                    .or_else(|| self.param_origin(id))
                    .or_else(|| self.owned_binding(id, depth))
                    .unwrap_or_else(PathOrigin::unknown);
                if origin.param.is_none()
                    && origin.env.is_none()
                    && origin.anchor == PathAnchor::Unknown
                {
                    origin.reassigned_at = self.reassigned_parameter(id);
                }
                origin
            }
        }
    }

    /// Origin of a path expression with no call-site context.
    pub(super) fn path_of(&mut self, expression: &Expression<'a>) -> PathOrigin {
        self.origin(expression, 0)
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
