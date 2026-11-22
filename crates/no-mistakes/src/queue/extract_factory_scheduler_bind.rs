use crate::fx::FxHashMap;
use oxc_ast::ast::{
    BindingPattern, Expression, FormalParameters, VariableDeclarationKind, VariableDeclarator,
};
use oxc_span::Span;
use oxc_syntax::scope::ScopeId;
use std::cell::{Cell, OnceCell, RefCell};

pub struct ObjectBinding<'a> {
    pub name: &'a str,
    pub scope_start: u32,
    pub scope_end: u32,
    pub visible_from: u32,
    pub resolve_at: u32,
    pub value: Option<&'a Expression<'a>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Function,
    Arrow,
    Block,
}

struct Scope {
    start: u32,
    end: u32,
    kind: ScopeKind,
}

type BindingsByName<'a> = FxHashMap<&'a str, Vec<ObjectBinding<'a>>>;

#[derive(Default)]
pub(super) struct ObjectBindings<'a> {
    by_name: OnceCell<BindingsByName<'a>>,
    declarations: RefCell<Vec<ObjectBinding<'a>>>,
    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(super) program_visits: usize,
}

impl<'a> ObjectBindings<'a> {
    pub(super) fn lookup(&self, name: &str, at: u32) -> Option<&ObjectBinding<'a>> {
        let binding = self
            .named(name)
            .iter()
            .filter(|binding| binding.scope_start <= at && at < binding.scope_end)
            .max_by_key(|binding| binding.scope_start)?;
        (at >= binding.visible_from).then_some(binding)
    }

    pub(super) fn named(&self, name: &str) -> &[ObjectBinding<'a>] {
        self.by_name
            .get_or_init(|| {
                let mut by_name = BindingsByName::default();
                for binding in self.declarations.take() {
                    by_name.entry(binding.name).or_default().push(binding);
                }
                by_name
            })
            .get(name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    pub(super) fn was_indexed(&self) -> bool {
        self.by_name.get().is_some()
    }

    fn insert(&mut self, binding: ObjectBinding<'a>) {
        self.declarations.get_mut().push(binding);
    }
}

#[derive(Default)]
pub(super) struct Collector<'a> {
    pub(super) bindings: ObjectBindings<'a>,
    scopes: Vec<Scope>,
    switches: Vec<(&'a Cell<Option<ScopeId>>, Span)>,
}

impl<'a> Collector<'a> {
    fn current(&self) -> &Scope {
        self.scopes.last().expect("binding scope")
    }

    fn var_span(&self) -> (u32, u32) {
        let scope = self
            .scopes
            .iter()
            .rev()
            .find(|scope| matches!(scope.kind, ScopeKind::Function | ScopeKind::Arrow))
            .expect("function scope");
        (scope.start, scope.end)
    }

    fn push_scope(&mut self, start: u32, end: u32, kind: ScopeKind) {
        self.scopes.push(Scope { start, end, kind });
    }

    fn mask(&mut self, name: &'a str, visible_from: u32) {
        let scope = self.current();
        self.bindings.insert(ObjectBinding {
            name,
            scope_start: scope.start,
            scope_end: scope.end,
            visible_from,
            resolve_at: visible_from,
            value: None,
        });
    }

    fn mask_pattern(&mut self, pattern: &BindingPattern<'a>, visible_from: u32) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => self.mask(id.name.as_str(), visible_from),
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.mask_pattern(&property.value, visible_from);
                }
                if let Some(rest) = &object.rest {
                    self.mask_pattern(&rest.argument, visible_from);
                }
            }
            BindingPattern::ArrayPattern(array) => {
                for element in array.elements.iter().flatten() {
                    self.mask_pattern(element, visible_from);
                }
                if let Some(rest) = &array.rest {
                    self.mask_pattern(&rest.argument, visible_from);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => {
                self.mask_pattern(&assignment.left, visible_from);
            }
        }
    }

    fn mask_params(&mut self, params: &FormalParameters<'a>, visible_from: u32) {
        for param in &params.items {
            self.mask_pattern(&param.pattern, visible_from);
        }
        if let Some(rest) = &params.rest {
            self.mask_pattern(&rest.rest.argument, visible_from);
        }
    }

    fn record_declarator(
        &mut self,
        kind: VariableDeclarationKind,
        declarator: &'a VariableDeclarator<'a>,
    ) {
        let module = self.scopes.len() == 1;
        let (var_start, var_end) = self.var_span();
        let current = self.current();
        let (scope_start, scope_end, visible_from) = if kind == VariableDeclarationKind::Var {
            (var_start, var_end, var_start)
        } else if module {
            (current.start, current.end, current.start)
        } else {
            (current.start, current.end, declarator.span.end)
        };
        let value = if kind == VariableDeclarationKind::Const {
            match &declarator.id {
                BindingPattern::BindingIdentifier(_) => declarator
                    .init
                    .as_ref()
                    .map(crate::codebase::ts_source::unwrap_ts_wrappers)
                    .filter(|expr| {
                        matches!(
                            expr,
                            Expression::ObjectExpression(_) | Expression::Identifier(_)
                        )
                    }),
                _ => None,
            }
        } else {
            None
        };
        visit::bind_pattern(
            self,
            &declarator.id,
            scope_start,
            scope_end,
            visible_from,
            declarator.span.end.saturating_sub(1),
            value,
        );
    }
}

#[path = "extract_factory_scheduler_visit.rs"]
mod visit;
