use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Expression, FormalParameters, Program, VariableDeclarationKind,
    VariableDeclarator,
};
use oxc_ast_visit::Visit;

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

pub fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
    allocator: &'a Allocator,
) -> Vec<ObjectBinding<'a>> {
    let mut collector = Collector {
        allocator,
        bindings: Vec::new(),
        scopes: Vec::new(),
    };
    collector.visit_program(program);
    collector.bindings
}

pub(super) fn lookup<'bindings, 'a>(
    bindings: &'bindings [ObjectBinding<'a>],
    name: &str,
    at: u32,
) -> Option<&'bindings ObjectBinding<'a>> {
    let mut best: Option<&ObjectBinding<'_>> = None;
    for binding in bindings {
        if binding.name != name || at < binding.scope_start || at >= binding.scope_end {
            continue;
        }
        if best.is_none_or(|current| binding.scope_start >= current.scope_start) {
            best = Some(binding);
        }
    }
    let binding = best?;
    (at >= binding.visible_from).then_some(binding)
}

struct Collector<'a> {
    allocator: &'a Allocator,
    bindings: Vec<ObjectBinding<'a>>,
    scopes: Vec<Scope>,
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
            .find(|scope| scope.kind == ScopeKind::Function)
            .expect("function scope");
        (scope.start, scope.end)
    }

    fn push_scope(&mut self, start: u32, end: u32, kind: ScopeKind) {
        self.scopes.push(Scope { start, end, kind });
    }

    fn mask(&mut self, name: &'a str, visible_from: u32) {
        let scope = self.current();
        self.bindings.push(ObjectBinding {
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
        declarator: &VariableDeclarator<'a>,
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
                    .and_then(|expr| visit::stored_expr(expr, self.allocator)),
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
