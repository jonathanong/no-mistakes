use super::expr::{call_member, pattern_names, unwrap_expr};
use super::extract::ExtractCtx;
use super::item::{ItemUse, ListingItem};
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::object_bindings::collect_names;
use oxc_ast::ast::{
    ArrowFunctionExpression, BlockStatement, CallExpression, Expression, ForInStatement,
    ForOfStatement, ForStatement, ForStatementLeft, Function, IdentifierReference, Statement,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};

pub(super) enum ActionScope<'a> {
    StateGuard(Vec<String>),
    Listing(ListingItem<'a>),
}

pub(super) enum Action {
    Retry,
    Enqueue(ProducerBinding),
    Uncertain(ProducerBinding),
}

impl Action {
    pub(super) fn into_site(self) -> (Option<ProducerBinding>, bool) {
        match self {
            Self::Retry => (None, false),
            Self::Enqueue(binding) => (Some(binding), false),
            Self::Uncertain(binding) => (Some(binding), true),
        }
    }
}

impl ActionScope<'_> {
    fn matches(&self, visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> bool {
        match self {
            Self::StateGuard(names) => names.iter().any(|name| name == id.name.as_str()),
            Self::Listing(item) => item.matches(visitor, id),
        }
    }

    fn enqueue(
        &self,
        visitor: &QueueVisitor<'_>,
        call: &CallExpression<'_>,
        binding: ProducerBinding,
    ) -> Option<Action> {
        match self {
            Self::StateGuard(_) => Some(Action::Enqueue(binding)),
            Self::Listing(item) => match item.enqueue_use(visitor, call) {
                ItemUse::Absent => None,
                ItemUse::Uncertain => Some(Action::Uncertain(binding)),
                ItemUse::Proven => Some(Action::Enqueue(binding)),
            },
        }
    }
}

pub(super) fn find_in_expr(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        ctx,
        scope,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

pub(super) fn find_in_stmts(
    ctx: &mut ExtractCtx<'_, '_>,
    stmts: &[Statement<'_>],
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        ctx,
        scope,
        found: None,
    };
    for stmt in stmts {
        finder.visit_statement(stmt);
        if finder.proven() {
            break;
        }
    }
    finder.found
}

struct ActionFinder<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
    scope: ActionScope<'a>,
    found: Option<Action>,
}

impl ActionFinder<'_, '_, '_> {
    fn proven(&self) -> bool {
        matches!(self.found, Some(Action::Retry | Action::Enqueue(_)))
    }

    fn with_scope(&mut self, walk_inner: impl FnOnce(&mut Self)) {
        let saved = match &self.scope {
            ActionScope::StateGuard(names) => Some(names.clone()),
            ActionScope::Listing(_) => None,
        };
        walk_inner(self);
        if let (ActionScope::StateGuard(names), Some(saved)) = (&mut self.scope, saved) {
            *names = saved;
        }
    }
}

impl<'a> Visit<'a> for ActionFinder<'_, '_, '_> {
    fn visit_function(&mut self, _func: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.with_scope(|this| walk::walk_block_statement(this, block));
    }
    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        self.with_scope(|this| walk::walk_for_statement(this, stmt));
    }
    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        self.with_scope(|this| walk::walk_for_of_statement(this, stmt));
    }
    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        self.with_scope(|this| walk::walk_for_in_statement(this, stmt));
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        let Some(init) = &decl.init else {
            return;
        };
        self.visit_expression(init);
        if let ActionScope::StateGuard(job_names) = &mut self.scope {
            let names = pattern_names(&decl.id);
            job_names.retain(|name| !names.contains(name));
            if let Expression::Identifier(src) = unwrap_expr(init) {
                if job_names.iter().any(|name| name == src.name.as_str()) {
                    job_names.extend(names);
                }
            }
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.proven() {
            return;
        }
        match call.callee.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) => {
                self.visit_arrow_function_body(&arrow.body);
                return;
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    self.visit_function_body(body);
                }
                return;
            }
            _ => {}
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry"
                || (method == "remove" && matches!(self.scope, ActionScope::StateGuard(_)));
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.scope.matches(self.ctx.visitor, id) {
                        self.found = Some(Action::Retry);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(binding) = self.ctx.queue_receiver(object, call.span.start) {
                    if let Some(action) = self.scope.enqueue(self.ctx.visitor, call, binding) {
                        self.found = Some(action);
                        if self.proven() {
                            return;
                        }
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

pub(super) fn for_of_names<'a>(left: &ForStatementLeft<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    if let ForStatementLeft::VariableDeclaration(decl) = left {
        for declaration in &decl.declarations {
            collect_names(&declaration.id, &mut names);
        }
    }
    names
}
