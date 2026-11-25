use super::expr::{call_member, queue_receiver, unwrap_expr};
use super::extract::FnEnv;
use super::item::{ItemUse, ListingItem};
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::object_bindings::collect_names;
use oxc_ast::ast::{CallExpression, Expression, ForStatementLeft, IdentifierReference, Statement};
use oxc_ast_visit::{walk, Visit};

#[derive(Clone, Copy)]
pub(super) enum ActionScope<'a> {
    StateGuard(&'a [&'a str]),
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
    fn matches(self, visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> bool {
        match self {
            Self::StateGuard(names) => names.contains(&id.name.as_str()),
            Self::Listing(item) => item.matches(visitor, id),
        }
    }

    fn enqueue(
        self,
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

pub(super) fn find_guarded_action(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    env: &FnEnv,
) -> Option<Action> {
    let names: Vec<_> = env.job_vars.keys().map(String::as_str).collect();
    find_action_in_stmt(visitor, stmt, ActionScope::StateGuard(&names))
}

pub(super) fn find_action_in_expr(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        visitor,
        scope,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

pub(super) fn find_action_in_stmt(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    scope: ActionScope<'_>,
) -> Option<Action> {
    find_action_in_stmts(visitor, std::slice::from_ref(stmt), scope)
}

pub(super) fn find_action_in_stmts(
    visitor: &QueueVisitor<'_>,
    stmts: &[Statement<'_>],
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        visitor,
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

struct ActionFinder<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    scope: ActionScope<'b>,
    found: Option<Action>,
}

impl ActionFinder<'_, '_> {
    fn proven(&self) -> bool {
        matches!(self.found, Some(Action::Retry | Action::Enqueue(_)))
    }
}

impl<'a> Visit<'a> for ActionFinder<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.proven() {
            return;
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry"
                || (method == "remove" && matches!(self.scope, ActionScope::StateGuard(_)));
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.scope.matches(self.visitor, id) {
                        self.found = Some(Action::Retry);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(binding) = queue_receiver(self.visitor, object, call.span.start) {
                    if let Some(action) = self.scope.enqueue(self.visitor, call, binding) {
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
