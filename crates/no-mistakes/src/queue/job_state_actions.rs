use super::expr::{call_member, queue_receiver, unwrap_expr};
use super::extract::FnEnv;
use crate::fx::FxHashSet;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::object_bindings::collect_names;
use oxc_ast::ast::{CallExpression, Expression, ForStatementLeft, IdentifierReference, Statement};
use oxc_ast_visit::{walk, Visit};

#[derive(Clone, Copy)]
pub(super) enum ActionScope<'a> {
    StateGuard(&'a [&'a str]),
    Listing { names: &'a [&'a str], start: u32 },
}

impl ActionScope<'_> {
    fn matches(self, visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> bool {
        let name = id.name.as_str();
        match self {
            Self::StateGuard(names) => names.contains(&name),
            Self::Listing { names, start } => {
                names.contains(&name)
                    && visitor
                        .object_bindings
                        .lookup(name, start)
                        .zip(visitor.object_bindings.lookup(name, id.span.start))
                        .is_some_and(|(item, used)| std::ptr::eq(item, used))
            }
        }
    }

    fn permits_enqueue(self, visitor: &QueueVisitor<'_>, call: &CallExpression<'_>) -> bool {
        let Self::Listing { start, .. } = self else {
            return true;
        };
        let mut uses_item = ItemReference {
            visitor,
            scope: self,
            start,
            found: false,
            aliases: Vec::new(),
            visited: FxHashSet::default(),
        };
        for arg in &call.arguments {
            uses_item.visit_argument(arg);
            if uses_item.found {
                return true;
            }
        }
        while let Some(alias) = uses_item.aliases.pop() {
            uses_item.visit_expression(alias);
            if uses_item.found {
                return true;
            }
        }
        false
    }
}

struct ItemReference<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    scope: ActionScope<'b>,
    start: u32,
    found: bool,
    aliases: Vec<&'c Expression<'c>>,
    visited: FxHashSet<u32>,
}

impl<'a> Visit<'a> for ItemReference<'_, '_> {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        if self.found {
            return;
        }
        if self.scope.matches(self.visitor, id) {
            self.found = true;
            return;
        }
        if let Some(binding) = self
            .visitor
            .object_bindings
            .lookup(id.name.as_str(), id.span.start)
            .filter(|binding| binding.scope_start >= self.start)
        {
            if let Some(alias) = binding.init.filter(|init| {
                matches!(
                    init,
                    Expression::Identifier(_)
                        | Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                )
            }) {
                if self.visited.insert(binding.resolve_at) {
                    self.aliases.push(alias);
                }
            }
        }
    }
}

pub(super) fn find_guarded_action(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    env: &FnEnv,
) -> Option<Option<ProducerBinding>> {
    let names: Vec<_> = env.job_vars.keys().map(String::as_str).collect();
    find_action_in_stmt(visitor, stmt, ActionScope::StateGuard(&names))
}

pub(super) fn find_action_in_expr(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    scope: ActionScope<'_>,
) -> Option<Option<ProducerBinding>> {
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
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder {
        visitor,
        scope,
        found: None,
    };
    finder.visit_statement(stmt);
    finder.found
}

struct ActionFinder<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    scope: ActionScope<'b>,
    found: Option<Option<ProducerBinding>>,
}

impl<'a> Visit<'a> for ActionFinder<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.found.is_some() {
            return;
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry"
                || (method == "remove" && matches!(self.scope, ActionScope::StateGuard(_)));
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.scope.matches(self.visitor, id) {
                        self.found = Some(None);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(binding) = queue_receiver(self.visitor, object, call.span.start) {
                    if self.scope.permits_enqueue(self.visitor, call) {
                        self.found = Some(Some(binding));
                        return;
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
