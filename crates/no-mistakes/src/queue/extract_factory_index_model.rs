use crate::fx::FxHashMap;
use oxc_ast::ast::{Argument, Expression};
use oxc_span::Span;

#[derive(Clone, Copy)]
pub(super) enum Binding {
    Other,
    Callable(Span),
    Factory(usize),
    Constructor,
}

#[derive(Default)]
pub(super) struct Index {
    pub scopes: Vec<FxHashMap<String, Binding>>,
    pub factories: FxHashMap<String, Binding>,
    pub parents: Vec<usize>,
    pub scope: usize,
    pub immutable: bool,
    pub writes: Vec<(usize, String)>,
    pub workers: Vec<(usize, String, bool, Vec<Option<String>>)>,
}

impl Index {
    pub fn insert(&mut self, name: &str, binding: Binding) {
        let bindings = &mut self.scopes[self.scope];
        let binding = if matches!(bindings.get(name), Some(Binding::Callable(_))) {
            Binding::Other
        } else {
            binding
        };
        bindings.insert(name.to_string(), binding);
    }

    pub fn lookup(&self, mut scope: usize, name: &str) -> Binding {
        loop {
            if let Some(binding) = self.scopes[scope].get(name) {
                return *binding;
            }
            if scope == 0 {
                return Binding::Other;
            }
            scope = self.parents[scope];
        }
    }

    pub fn worker(&mut self, callee: &Expression<'_>, args: &[Argument<'_>], constructed: bool) {
        if let Expression::Identifier(id) = callee {
            if !self.factories.contains_key(id.name.as_str()) {
                return;
            }
            let args = args
                .iter()
                .map(|arg| match arg.as_expression()?.get_inner_expression() {
                    Expression::Identifier(id) => Some(id.name.to_string()),
                    _ => None,
                })
                .collect();
            self.workers
                .push((self.scope, id.name.to_string(), constructed, args));
        }
    }

    pub fn invalidate_writes(&mut self) {
        for (mut scope, name) in std::mem::take(&mut self.writes) {
            loop {
                if let Some(binding) = self.scopes[scope].get_mut(&name) {
                    *binding = Binding::Other;
                    break;
                }
                if scope == 0 {
                    break;
                }
                scope = self.parents[scope];
            }
        }
    }
}
