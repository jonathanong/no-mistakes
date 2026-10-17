//! A `const` bound to a directly invoked function that builds and returns one
//! local SQL builder:
//!
//! ```ts
//! const built = (() => {
//!   const query = sql`...`;
//!   query.append(sql`...`);
//!   return query;
//! })();
//! ```
//!
//! The body is walked like any other function body. When it ends, the returned
//! builder's final state (text, positions and finite alternatives) is copied
//! to the `const`. Anything outside this exact shape keeps the `const` opaque.

use super::{BindingState, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, Expression, FormalParameters, FunctionBody, IdentifierReference,
    ReturnStatement, Statement, ThisExpression, VariableDeclarationKind, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};

/// A recognized invocation: the local it returns and that local's final state.
pub(super) struct Iife {
    pub(super) returned: String,
    pub(super) state: Option<BindingState>,
}

/// The shape check for `init`, or `None` when it is not a safe IIFE.
pub(super) fn target(init: &Expression<'_>) -> Option<Iife> {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(init) else {
        return None;
    };
    if call.optional || !call.arguments.is_empty() {
        return None;
    }
    let (params, body) = match unwrap_ts_wrappers(&call.callee) {
        Expression::ArrowFunctionExpression(arrow) if !arrow.r#async => {
            (&*arrow.params, arrow.body.as_function_body()?)
        }
        Expression::FunctionExpression(function)
            if !function.r#async && !function.generator && function.id.is_none() =>
        {
            (&*function.params, function.body.as_deref()?)
        }
        _ => return None,
    };
    no_parameters(params)?;
    let returned = sole_local_return(body)?;
    let mut scan = Scan::default();
    scan.visit_function_body(body);
    (scan.returns == 1 && !scan.forbidden).then_some(Iife {
        returned,
        state: None,
    })
}

fn no_parameters(params: &FormalParameters<'_>) -> Option<()> {
    (params.items.is_empty() && params.rest.is_none()).then_some(())
}

/// The body ends in `return <name>` and `name` is a `const`/`let` it declares.
fn sole_local_return(body: &FunctionBody<'_>) -> Option<String> {
    let (last, before) = body.statements.split_last()?;
    let Statement::ReturnStatement(statement) = last else {
        return None;
    };
    let Expression::Identifier(returned) = unwrap_ts_wrappers(statement.argument.as_ref()?) else {
        return None;
    };
    let declared = before.iter().any(|statement| {
        let Statement::VariableDeclaration(declaration) = statement else {
            return false;
        };
        declaration.kind != VariableDeclarationKind::Var
            && declaration.declarations.iter().any(|declarator| {
                matches!(
                    &declarator.id,
                    BindingPattern::BindingIdentifier(id) if id.name == returned.name
                )
            })
    });
    declared.then(|| returned.name.to_string())
}

/// Counts the body's own returns and flags `this` / `arguments`, which would
/// make the function depend on how it is invoked.
#[derive(Default)]
struct Scan {
    returns: usize,
    nested: usize,
    forbidden: bool,
}

impl<'a> Visit<'a> for Scan {
    fn visit_return_statement(&mut self, statement: &oxc_ast::ast::ReturnStatement<'a>) {
        if self.nested == 0 {
            self.returns += 1;
        }
        walk::walk_return_statement(self, statement);
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.nested += 1;
        walk::walk_function(self, function, flags);
        self.nested -= 1;
    }

    fn visit_arrow_function_expression(
        &mut self,
        arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        self.nested += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.nested -= 1;
    }

    fn visit_this_expression(&mut self, _: &ThisExpression) {
        self.forbidden = true;
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.forbidden |= identifier.name == "arguments";
    }
}

impl ScopeVisitor<'_> {
    /// Called at a `return`: when it returns the recognized IIFE's local,
    /// remember that builder's final state.
    pub(super) fn capture_iife(&mut self, statement: &ReturnStatement<'_>) {
        let Some(iife) = &self.iife else {
            return;
        };
        let returns_it = statement.argument.as_ref().is_some_and(|argument| {
            matches!(
                unwrap_ts_wrappers(argument),
                Expression::Identifier(id) if id.name.as_str() == iife.returned
            )
        });
        if returns_it {
            let state = self.lookup(&iife.returned.clone());
            if let Some(iife) = &mut self.iife {
                iife.state = state;
            }
        }
    }

    /// Starts walking a declarator's initializer; returns what to restore.
    pub(super) fn enter_iife(&mut self, declarator: &VariableDeclarator<'_>) -> Option<Iife> {
        // Only a `const` is proven to keep the value it was initialized with.
        let is_const = matches!(
            &declarator.id,
            BindingPattern::BindingIdentifier(id)
                if self.lookup(id.name.as_str()).is_some_and(|b| b.executor_alias.is_some())
        );
        let iife = declarator
            .init
            .as_ref()
            .filter(|_| is_const)
            .and_then(target);
        std::mem::replace(&mut self.iife, iife)
    }

    /// Finishes the declarator: a recognized IIFE hands its builder over.
    pub(super) fn leave_iife(&mut self, id: &BindingPattern<'_>, outer: Option<Iife>) {
        let captured = std::mem::replace(&mut self.iife, outer).and_then(|iife| iife.state);
        if let (Some(state), BindingPattern::BindingIdentifier(id)) = (captured, id) {
            self.adopt_iife(id.name.as_str(), state);
        }
    }

    /// Gives the `const` the captured builder's final state.
    pub(super) fn adopt_iife(&mut self, name: &str, captured: BindingState) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.get_mut(name) {
                binding.sql = captured.sql;
                binding.kind = captured.kind;
                binding.line = captured.line;
                binding.sql_builder = captured.sql_builder;
                binding.sql_source_positions = captured.sql_source_positions;
                binding.variants = captured.variants;
                binding.builder_identity = captured.builder_identity;
                binding.condition_key = None;
                binding.initialized = true;
                return;
            }
        }
    }
}
