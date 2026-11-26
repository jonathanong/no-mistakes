use super::QueueVisitor;
use crate::fx::{FxHashMap, FxHashSet};
use crate::queue::extract_job_id::referenced_symbol;
use oxc_ast::ast::{
    Argument, BindingIdentifier, BindingPattern, Expression, FormalParameters, Function,
    ThrowStatement, VariableDeclarator,
};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

mod exports;
mod finish;
mod imports;
mod locals;
use imports::{Import, ImportedMember, ThrowTarget};
use locals::{callable_start, ObjProp};

#[derive(Default)]
pub(in crate::queue) struct State {
    enabled: bool,
    imports: FxHashMap<SymbolId, Import>,
    named: FxHashMap<SymbolId, u32>,
    processors: Vec<(Processor, Owner)>,
    wrappers: Vec<(Span, Owner)>,
    parameters: FxHashMap<SymbolId, (u32, ImportedMember)>,
    uses: Vec<Use>,
    local_receivers: FxHashMap<u32, usize>,
    local_non_queue: FxHashSet<SymbolId>,
    local_aliases: FxHashMap<SymbolId, SymbolId>,
    local_object_rate_limit: FxHashMap<SymbolId, ObjProp>,
}

struct Owner {
    worker: usize,
    configured_factory: bool,
}

enum Processor {
    Named(SymbolId),
    Inline(u32),
}

struct Use {
    start: u32,
    functions: Vec<u32>,
    kind: UseKind,
}

enum UseKind {
    Throw(ThrowTarget),
    Call(Option<SymbolId>),
}

impl State {
    pub(in crate::queue) fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }
}

impl QueueVisitor<'_> {
    pub(in crate::queue) fn note_rate_limit_processor(
        &mut self,
        argument: &Argument<'_>,
        worker: usize,
        configured_factory: bool,
    ) {
        if !self.shared_rate_limit.enabled {
            return;
        }
        let Some(expr) = argument
            .as_expression()
            .map(Expression::get_inner_expression)
        else {
            return;
        };
        let owner = Owner {
            worker,
            configured_factory,
        };
        if let Expression::Identifier(id) = expr {
            if let Some(symbol) = referenced_symbol(self, id) {
                self.shared_rate_limit
                    .processors
                    .push((Processor::Named(symbol), owner));
            }
        } else if let Some(start) = callable_start(expr) {
            self.shared_rate_limit
                .processors
                .push((Processor::Inline(start), owner));
        } else if let Expression::CallExpression(call) = expr {
            self.shared_rate_limit.wrappers.push((call.span, owner));
        }
    }

    pub(super) fn note_rate_limit_const(&mut self, declaration: &VariableDeclarator<'_>) {
        if !self.shared_rate_limit.enabled {
            return;
        }
        if let (BindingPattern::BindingIdentifier(id), Some(start)) = (
            &declaration.id,
            declaration.init.as_ref().and_then(callable_start),
        ) {
            if let Some(symbol) = self.stable_rate_limit_symbol(id) {
                self.shared_rate_limit.named.insert(symbol, start);
            }
        }
        self.note_rate_limit_local_const(declaration);
    }

    pub(in crate::queue) fn note_rate_limit_function(&mut self, function: &Function<'_>) {
        if self.shared_rate_limit.enabled && function.is_function_declaration() {
            if let Some(symbol) = function
                .id
                .as_ref()
                .and_then(|id| self.stable_rate_limit_symbol(id))
            {
                self.shared_rate_limit
                    .named
                    .insert(symbol, function.span.start);
                self.shared_rate_limit.local_non_queue.insert(symbol);
            }
        }
        self.note_rate_limit_parameters(function.span, &function.params);
    }

    pub(super) fn stable_rate_limit_symbol(
        &self,
        binding: &BindingIdentifier<'_>,
    ) -> Option<SymbolId> {
        let scoping = self.scoping?;
        let symbol = binding.symbol_id.get()?;
        (!scoping.symbol_is_mutated(symbol) && scoping.symbol_redeclarations(symbol).is_empty())
            .then_some(symbol)
    }

    pub(in crate::queue) fn note_rate_limit_parameters(
        &mut self,
        span: Span,
        parameters: &FormalParameters<'_>,
    ) {
        if !self.shared_rate_limit.enabled {
            return;
        }
        let state = &mut self.shared_rate_limit;
        while state
            .wrappers
            .pop_if(|(wrapper, _)| wrapper.end < span.start)
            .is_some()
        {}
        if let Some((_, owner)) = state
            .wrappers
            .pop_if(|(wrapper, _)| wrapper.start <= span.start && span.end <= wrapper.end)
        {
            state
                .processors
                .push((Processor::Inline(span.start), owner));
        }
        self.note_worker_parameters(span.start, parameters);
    }

    pub(in crate::queue) fn note_rate_limit_call(&mut self, start: u32, object: &Expression<'_>) {
        if self.shared_rate_limit.enabled && !self.function_stack.is_empty() {
            let symbol = match object.get_inner_expression() {
                Expression::Identifier(id) => referenced_symbol(self, id),
                _ => None,
            };
            self.note_rate_limit_use(start, UseKind::Call(symbol));
        }
    }

    pub(super) fn note_rate_limit_throw(&mut self, stmt: &ThrowStatement<'_>) {
        if self.shared_rate_limit.enabled && !self.function_stack.is_empty() {
            if let Some(target) = self.rate_limit_error(&stmt.argument) {
                self.note_rate_limit_use(stmt.span.start, UseKind::Throw(target));
            }
        }
    }

    fn note_rate_limit_use(&mut self, start: u32, kind: UseKind) {
        self.shared_rate_limit.uses.push(Use {
            start,
            functions: self.function_stack.clone(),
            kind,
        });
    }

    pub(in crate::queue) fn note_shared_rate_limit_receiver(&mut self, start: u32, worker: usize) {
        self.shared_rate_limit.local_receivers.insert(start, worker);
    }
}
