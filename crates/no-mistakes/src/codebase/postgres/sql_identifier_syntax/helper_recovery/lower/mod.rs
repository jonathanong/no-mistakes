//! Low-only closed lowering. Names in the shared IR are semantic binding slots.
mod calls;
mod expressions;
mod forms;
mod parameters;
mod statements;
mod templates;
use super::super::{span, template_binds::TemplateBinds, Span};
use crate::codebase::postgres::embedded::helpers::program::{Expr, Mark, Program, Stmt};
use crate::codebase::ts_source::{byte_offset_to_line, module_facts::TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{Expression, Function};
use oxc_span::GetSpan;

/// Shared per-file binding lookup, built from the already prepared semantics.
#[derive(Default)]
pub(crate) struct Bindings {
    slots: FxHashMap<Span, usize>,
}
impl Bindings {
    pub fn new(module: &TypeScriptModuleFacts) -> Self {
        let mut slots = FxHashMap::default();
        for binding in &module.bindings {
            slots.insert((binding.span.start, binding.span.end), binding.id);
            for reference in &binding.references {
                if reference.runtime {
                    slots.insert((reference.span.start, reference.span.end), binding.id);
                }
            }
        }
        Self { slots }
    }
    pub fn id(&self, at: oxc_span::Span) -> Option<usize> {
        self.slots.get(&span(at)).copied()
    }
    pub fn name(&self, at: oxc_span::Span) -> Option<String> {
        self.id(at).map(|id| format!("binding:{id}"))
    }
}
#[derive(Clone)]
pub(crate) struct Lowered {
    pub program: Program,
    pub asynchronous: bool,
    /// Every library tag used by this model must survive whole-file writes.
    pub tags: FxHashSet<usize>,
    /// Exact binding IDs, never a textual lookup in the helper's caller.
    pub references: Vec<(usize, Span)>,
}
pub(crate) struct Context<'a> {
    pub source: &'a str,
    pub bindings: &'a Bindings,
    pub(in crate::codebase::postgres::sql_identifier_syntax) tags: &'a TemplateBinds,
}
struct Scope<'a> {
    ctx: &'a Context<'a>,
    tags: FxHashSet<usize>,
    work: usize,
    references: Vec<(usize, Span)>,
}
impl Scope<'_> {
    fn mark(&self, at: oxc_span::Span) -> Mark {
        Mark {
            line: byte_offset_to_line(self.ctx.source, at.start as usize),
            origin: at.start,
        }
    }
    fn expression(&mut self, node: &Expression<'_>) -> Expr {
        self.work += 1;
        if self.work > 10_000 {
            return Expr::Opaque;
        }
        expressions::lower(self, node)
    }
}
/// Generators do not execute on call. Async results retain an explicit Promise.
pub(crate) fn function(function: &Function<'_>, ctx: &Context<'_>) -> Option<Lowered> {
    if function.generator || function.params.rest.is_some() {
        return None;
    }
    let mut scope = Scope {
        ctx,
        tags: FxHashSet::default(),
        work: 1,
        references: Vec::new(),
    };
    let (params, mut body) = parameters::lower(&mut scope, &function.params)?;
    body.extend(statements::block(
        &mut scope,
        &function.body.as_ref()?.statements,
    )?);
    Some(Lowered {
        program: Program { params, body },
        asynchronous: function.r#async,
        tags: scope.tags,
        references: scope.references,
    })
}

/// A SQL operand or helper argument evaluated under caller scalar bindings.
pub(crate) fn expression(node: &Expression<'_>, ctx: &Context<'_>) -> Lowered {
    let mut scope = Scope {
        ctx,
        tags: FxHashSet::default(),
        work: 0,
        references: Vec::new(),
    };
    let mark = scope.mark(node.span());
    let value = scope.expression(node);
    Lowered {
        program: Program {
            params: Vec::new(),
            body: vec![Stmt::Return(Some(value), mark)],
        },
        asynchronous: false,
        tags: scope.tags,
        references: scope.references,
    }
}
