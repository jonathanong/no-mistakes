//! Lowers a top-level helper's AST to a [`Program`].

mod compound;
mod expr;
mod stmt;
mod switch;
mod template;

pub(crate) use switch::names as throwing_names;

use super::{Mark, Param, Program, Stmt};
use crate::codebase::postgres::embedded::tags::SqlTagNames;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{BindingPattern, FormalParameters, FunctionBody, TSType, TSTypeName};
use oxc_span::GetSpan;
use std::collections::HashSet;

/// What lowering reads about the declaring file.
pub(crate) struct Ctx<'a> {
    pub(crate) source: &'a str,
    pub(crate) tags: &'a SqlTagNames,
    /// Local names of the `SQLStatement` type.
    pub(crate) builder_types: &'a HashSet<String>,
    /// Names that are not the trusted tag even when spelled like it
    /// (top-level rebindings; the function's own parameters are added here).
    pub(crate) rebound: &'a dyn Fn(&str) -> bool,
    /// Same-file functions whose bodies always throw.
    pub(crate) throws: &'a HashSet<String>,
}

/// The body of a callable: statements, or an arrow function's expression.
pub(crate) enum Body<'a, 'b> {
    Statements(&'b FunctionBody<'a>),
    Expression(&'b oxc_ast::ast::Expression<'a>),
}

/// Lowering state for one function.
pub(super) struct Scope<'c, 'a> {
    pub(super) ctx: &'c Ctx<'a>,
    /// Parameter names, which can shadow the trusted tag.
    pub(super) params: Vec<String>,
    /// Names declared so far in the function (parameters and locals).
    pub(super) declared: Vec<String>,
    /// A nested function declaration was seen; see `Program::body`.
    pub(super) hoisted: bool,
}

impl Scope<'_, '_> {
    pub(super) fn is_rebound(&self, name: &str) -> bool {
        self.declared.iter().any(|declared| declared == name) || (self.ctx.rebound)(name)
    }

    pub(super) fn mark(&self, span: oxc_span::Span) -> Mark {
        let origin = span.start;
        Mark {
            line: byte_offset_to_line(self.ctx.source, origin as usize),
            origin,
        }
    }
}

/// `None` for a parameter list the model cannot bind (destructuring, rest,
/// duplicate names). Statements it cannot model lower to `Opaque` instead.
pub(crate) fn lower(
    params: &FormalParameters<'_>,
    body: Body<'_, '_>,
    ctx: &Ctx<'_>,
) -> Option<Program> {
    if params.rest.is_some() {
        return None;
    }
    let mut scope = Scope {
        ctx,
        params: Vec::new(),
        declared: Vec::new(),
        hoisted: false,
    };
    for param in &params.items {
        let BindingPattern::BindingIdentifier(ident) = &param.pattern else {
            return None;
        };
        let name = ident.name.to_string();
        if scope.declared.contains(&name) {
            return None;
        }
        scope.declared.push(name.clone());
        scope.params.push(name);
    }
    let mut lowered = Vec::with_capacity(params.items.len());
    for (param, name) in params.items.iter().zip(scope.params.clone()) {
        let default = param
            .initializer
            .as_ref()
            .map(|default| expr::lower(&mut scope, default));
        let builder = is_builder_type(param, ctx.builder_types);
        lowered.push(Param {
            name,
            builder,
            default,
        });
    }
    let mut body = match body {
        Body::Statements(body) => stmt::block(&mut scope, &body.statements)?,
        Body::Expression(expression) => {
            let mark = scope.mark(expression.span());
            vec![Stmt::Return(
                Some(expr::lower(&mut scope, expression)),
                mark,
            )]
        }
    };
    if scope.hoisted {
        // Runs first, so a hoisted function is never silently skipped.
        body.insert(0, Stmt::Opaque);
    }
    Some(Program {
        params: lowered,
        body,
    })
}

impl Program {
    /// Whether any reachable-looking node can contribute SQL text.
    pub(crate) fn contributes_sql(&self) -> bool {
        self.body.iter().any(stmt::contributes_sql)
    }
}

fn is_builder_type(param: &oxc_ast::ast::FormalParameter<'_>, types: &HashSet<String>) -> bool {
    let Some(annotation) = &param.type_annotation else {
        return false;
    };
    let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
        return false;
    };
    matches!(
        &reference.type_name,
        TSTypeName::IdentifierReference(ty) if types.contains(ty.name.as_str())
    )
}
