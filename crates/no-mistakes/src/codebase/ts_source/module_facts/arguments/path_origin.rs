//! Static path provenance; classic temporary factories require Node bindings.
//! Path helpers use syntactic heuristics; unsupported expressions remain unknown.
mod alternatives;
mod argument_object;
mod bindings;
mod callbacks;
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
            Expression::AwaitExpression(inner) => self
                .awaited_promise_directory(&inner.argument)
                .unwrap_or_else(|| self.origin(&inner.argument, depth + 1)),
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

    /// `await` of an `fs/promises` `mkdtemp` call, or of an immutable `const`
    /// alias of that call, is the directory string. Another use of a mutable
    /// binding leaves it unproven. Awaiting a synchronous string is left to
    /// the inner origin.
    fn awaited_promise_directory(&mut self, argument: &Expression<'a>) -> Option<PathOrigin> {
        let mut bindings = Vec::new();
        let call = match argument.get_inner_expression() {
            Expression::CallExpression(call) => call,
            Expression::Identifier(id) => self.const_promise_call(id, &mut bindings)?,
            _ => return None,
        };
        if self.temp_factory_result(call.callee.get_inner_expression())
            != Some(temp_factories::TempReturn::Promise)
        {
            return None;
        }
        let mut origin = composite::temp_directory(call);
        for id in &bindings {
            origin = self.value_requirement(id, origin);
            if origin.anchor != PathAnchor::Temp {
                return None;
            }
        }
        Some(origin)
    }

    /// Call behind at most 16 immutable `const` identifiers, recording each hop.
    fn const_promise_call<'b>(
        &mut self,
        id: &'b IdentifierReference<'a>,
        bindings: &mut Vec<&'b IdentifierReference<'a>>,
    ) -> Option<&'a CallExpression<'a>> {
        bindings.push(id);
        let mut init = self.const_init(id)?;
        loop {
            match init.get_inner_expression() {
                Expression::CallExpression(call) => return Some(call),
                Expression::Identifier(alias) if bindings.len() < 16 => {
                    bindings.push(alias);
                    init = self.const_init(alias)?;
                }
                _ => return None,
            }
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
