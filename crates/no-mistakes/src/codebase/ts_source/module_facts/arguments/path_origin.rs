//! Static path provenance; classic temporary factories require Node bindings.
//! Path helpers use syntactic heuristics; unsupported expressions remain unknown.
mod alternatives;
mod argument_object;
mod bindings;
mod calls;
mod composite;
mod defaults;
mod destructure;
mod forwarded;
mod getters;
mod identifier;
mod member_effects;
mod member_mutations;
mod members;
mod nested_reads;
mod nullish;
mod object_paths;
mod object_rest;
mod origins;
mod patterns;
mod positions;
mod sources;
mod temp_factories;
mod tmpdir;

pub(super) use self::composite::{callee_root, imported_locals};
pub(super) use self::sources::{env_name, is_process_env, process_global};
pub(super) use self::temp_factories::{temp_factories, TempFactories};
use super::{PathAnchor, PathOrigin, PathValueKind, Projector};
use oxc_ast::{ast::*, AstKind};

impl<'a> Projector<'a, '_> {
    fn origin(&mut self, expression: &Expression<'a>, depth: usize) -> PathOrigin {
        if depth >= 16 || !self.take() {
            return PathOrigin::unknown();
        }
        match expression.get_inner_expression() {
            Expression::NullLiteral(_) => PathOrigin {
                nullish: true,
                value_kind: PathValueKind::Scalar,
                ..PathOrigin::unknown()
            },
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
                let mut left = self.origin(&logical.left, depth + 1);
                if left.env.is_some() {
                    if let (Some(hint), Expression::StringLiteral(value)) = (
                        &mut left.env_diagnostic,
                        logical.right.get_inner_expression(),
                    ) {
                        hint.fallback = Some(crate::ast::js_literal(value).into_owned());
                    }
                    left
                } else {
                    let right = self.origin(&logical.right, depth + 1);
                    if logical.operator == LogicalOperator::Coalesce {
                        left.coalesced(right)
                    } else {
                        PathOrigin::alternatives(vec![left, right])
                    }
                }
            }
            Expression::ChainExpression(chain) => self.chain_origin(chain, depth),
            Expression::NewExpression(url) => self.url(url, depth),
            Expression::CallExpression(call) => self.call(call, depth),
            _ => PathOrigin::unknown(),
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
