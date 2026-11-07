use super::ScopeVisitor;
use oxc_ast::ast::Statement;
use std::num::NonZeroU32;

type Identities = Vec<Vec<(String, Option<NonZeroU32>)>>;
pub(super) struct Snapshot {
    enabled: bool,
    identities: Identities,
    exited: crate::fx::FxHashSet<u32>,
}
impl Snapshot {
    pub(super) fn capture(visitor: &ScopeVisitor<'_>) -> Self {
        if !visitor.executor_options.track_transaction_helpers {
            return Self {
                enabled: false,
                identities: Vec::new(),
                exited: Default::default(),
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
        let mut live = false;
        let all_ended = self
            .identities
            .iter()
            .enumerate()
            .all(|(index, identities)| {
                identities.iter().all(|(name, identity)| {
                    if identity.is_none() {
                        return true;
                    }
                    live = true;
                    ended
                        .identities
                        .get(index)
                        .and_then(|scope| scope.iter().find(|(key, _)| key == name))
                        .is_some_and(|(_, ended)| ended != identity)
                })
            });
        if !live || !all_ended {
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
