use super::ScopeVisitor;
use oxc_ast::ast::Statement;
use std::num::NonZeroU32;

pub(super) type Completed = crate::fx::FxHashSet<(u32, NonZeroU32)>;

type Identities = Vec<Vec<(String, Option<NonZeroU32>)>>;
pub(super) struct Snapshot {
    enabled: bool,
    identities: Identities,
    exited: crate::fx::FxHashSet<u32>,
    incomplete: crate::fx::FxHashSet<u32>,
    completed: Completed,
}
impl Snapshot {
    pub(super) fn capture(visitor: &ScopeVisitor<'_>) -> Self {
        if !visitor.executor_options.track_transaction_helpers
            || visitor.scoped.transactions.is_empty()
        {
            return Self {
                enabled: false,
                identities: Vec::new(),
                exited: Default::default(),
                incomplete: Default::default(),
                completed: Default::default(),
            };
        }
        Self {
            enabled: true,
            identities: visitor
                .scopes
                .iter()
                .map(|scope| {
                    scope
                        .iter()
                        .filter(|(name, _)| visitor.scoped.transactions.contains_key(*name))
                        .map(|(name, binding)| (name.clone(), binding.value_identity))
                        .collect()
                })
                .collect(),
            exited: visitor.exited_functions.clone(),
            incomplete: visitor.incomplete_transaction_owners.clone(),
            completed: visitor.transaction_options.completed.clone(),
        }
    }
    pub(super) fn restore(&self, visitor: &mut ScopeVisitor<'_>) {
        if !self.enabled {
            return;
        }
        for (scope, identities) in visitor.scopes.iter_mut().zip(&self.identities) {
            for (name, identity) in identities {
                if let Some(binding) = scope.get_mut(name) {
                    binding.value_identity = *identity;
                }
            }
        }
        visitor.exited_functions = self.exited.clone();
        visitor.incomplete_transaction_owners = self.incomplete.clone();
        visitor.transaction_options.completed = self.completed.clone();
    }
    pub(super) fn resume_after(
        &self,
        ended: &Self,
        continuing: &Self,
        visitor: &mut ScopeVisitor<'_>,
    ) {
        continuing.restore(visitor);
        if !self.enabled {
            return;
        }
        let owner = owner(visitor);
        let mut live = false;
        let all_ended = self.identities.iter().flatten().all(|(_, identity)| {
            let Some(identity) = *identity else {
                return true;
            };
            live = true;
            ended.completed.contains(&(owner, identity))
        });
        if !live || !all_ended || ended.incomplete.contains(&owner) {
            visitor
                .exited_functions
                .extend(ended.exited.iter().copied());
        }
    }
    pub(super) fn join(&self, other: &Self, visitor: &mut ScopeVisitor<'_>) {
        if !self.enabled {
            return;
        }
        self.restore(visitor);
        for (scope, identities) in visitor.scopes.iter_mut().zip(&other.identities) {
            for (name, identity) in identities {
                if let Some(binding) = scope.get_mut(name) {
                    if binding.value_identity != *identity {
                        binding.value_identity = None;
                    }
                }
            }
        }
        visitor
            .transaction_options
            .completed
            .retain(|entry| other.completed.contains(entry));
        visitor
            .incomplete_transaction_owners
            .extend(other.incomplete.iter().copied());
        visitor
            .exited_functions
            .extend(other.exited.iter().copied());
    }
}
pub(super) fn exits(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => true,
        Statement::BlockStatement(block) => block.body.last().is_some_and(exits),
        Statement::IfStatement(statement) => {
            exits(&statement.consequent) && statement.alternate.as_ref().is_some_and(exits)
        }
        _ => false,
    }
}

pub(super) fn note_boundary(
    visitor: &mut ScopeVisitor<'_>,
    call: &oxc_ast::ast::CallExpression<'_>,
    boundary: bool,
) {
    if boundary
        && visitor.executor_options.track_transaction_helpers
        && !visitor.awaited_lock_helper_calls.contains(&call.span.start)
    {
        visitor.incomplete_transaction_owners.insert(owner(visitor));
    }
}

fn owner(visitor: &ScopeVisitor<'_>) -> u32 {
    visitor
        .member_query_proof
        .function_ids
        .last()
        .copied()
        .unwrap_or(0)
}

pub(super) fn complete_boundary(
    visitor: &mut ScopeVisitor<'_>,
    call: &oxc_ast::ast::CallExpression<'_>,
    identity: Option<NonZeroU32>,
) {
    if visitor.executor_options.track_transaction_helpers
        && visitor.loop_depth == 0
        && visitor.transaction_options.suspended == 0
        && visitor.control_depth == visitor.transaction_options.if_depth
        && visitor.awaited_lock_helper_calls.contains(&call.span.start)
    {
        if let Some(identity) = identity {
            visitor
                .transaction_options
                .completed
                .insert((owner(visitor), identity));
        }
    }
}
