use oxc_ast::ast::{
    BindingPattern, Expression, ForStatementLeft, VariableDeclarationKind, VariableDeclarator,
};
use oxc_span::Span;
use oxc_syntax::scope::ScopeId;
use std::cell::Cell;

#[path = "object_binding_index.rs"]
mod index;
pub(in crate::queue) use index::ObjectBindings;
pub(in crate::queue) use visit::collect_names;

pub struct ObjectBinding<'a> {
    pub name: &'a str,
    pub scope_start: u32,
    pub scope_end: u32,
    pub scope_depth: usize,
    bare_var: bool,
    pub visible_from: u32,
    pub resolve_at: u32,
    pub init: Option<&'a Expression<'a>>,
    pub destructuring: Option<(&'a BindingPattern<'a>, &'a Expression<'a>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    Function,
    Arrow,
    Block,
}

#[derive(Clone, Copy)]
struct Scope {
    start: u32,
    end: u32,
    kind: ScopeKind,
    depth: usize,
}

impl<'a> ObjectBinding<'a> {
    pub fn value(&self) -> Option<&'a Expression<'a>> {
        self.init.filter(|expr| {
            matches!(
                expr.get_inner_expression(),
                Expression::ObjectExpression(_) | Expression::Identifier(_)
            )
        })
    }

    pub fn in_scope<'b>(bindings: &'b [Self], name: &str, at: u32) -> Option<&'b Self> {
        bindings
            .iter()
            .filter(|binding| {
                binding.name == name && binding.scope_start <= at && at < binding.scope_end
            })
            .max_by_key(|binding| (binding.scope_depth, !binding.bare_var, binding.scope_start))
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

    fn var_scope(&self) -> Scope {
        *self
            .scopes
            .iter()
            .rev()
            .find(|scope| matches!(scope.kind, ScopeKind::Function | ScopeKind::Arrow))
            .expect("function scope")
    }

    fn push_scope(&mut self, start: u32, end: u32, kind: ScopeKind) {
        let depth = self.scopes.len();
        self.scopes.push(Scope {
            start,
            end,
            kind,
            depth,
        });
    }

    fn mask(&mut self, name: &'a str, visible_from: u32) {
        self.mask_in_scope(name, *self.current(), visible_from);
    }

    fn mask_in_scope(&mut self, name: &'a str, scope: Scope, visible_from: u32) {
        self.bindings.insert(ObjectBinding {
            name,
            scope_start: scope.start,
            scope_end: scope.end,
            scope_depth: scope.depth,
            bare_var: false,
            visible_from,
            resolve_at: visible_from,
            init: None,
            destructuring: None,
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

    fn mask_iterated_vars(&mut self, left: &ForStatementLeft<'a>, body_start: u32) {
        let ForStatementLeft::VariableDeclaration(decl) = left else {
            return;
        };
        if decl.kind != VariableDeclarationKind::Var {
            return;
        }
        let mut scope = self.var_scope();
        scope.start = body_start;
        let mut names = Vec::new();
        for declarator in &decl.declarations {
            visit::collect_names(&declarator.id, &mut names);
        }
        for name in names {
            self.mask_in_scope(name, scope, body_start);
        }
    }

    fn record_declarator(
        &mut self,
        kind: VariableDeclarationKind,
        declarator: &'a VariableDeclarator<'a>,
    ) {
        let var = kind == VariableDeclarationKind::Var;
        let scope = if var {
            self.var_scope()
        } else {
            *self.current()
        };
        let visible_from = if var || scope.depth == 0 {
            scope.start
        } else {
            declarator.span.end
        };
        let init = (kind == VariableDeclarationKind::Const)
            .then_some(declarator.init.as_ref())
            .flatten()
            .map(crate::codebase::ts_source::unwrap_ts_wrappers);
        visit::bind_pattern(
            self,
            &declarator.id,
            scope,
            visible_from,
            declarator.span.end.saturating_sub(1),
            var && declarator.init.is_none(),
            init,
        );
    }
}

#[path = "object_bindings_visit.rs"]
mod visit;

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "object_bindings_test_hook.rs"]
pub(super) mod test_hook;
