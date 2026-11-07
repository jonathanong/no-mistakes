mod boundary;
pub(in crate::codebase::postgres::embedded::walk::transaction) use boundary::end_alias;

use super::{receiver, value_receiver, EmbeddedSqlCall, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Expression, IdentifierReference, VariableDeclarator};
use std::{cell::RefCell, num::NonZeroU32};

#[derive(Clone)]
struct Captured {
    name: String,
    identity: NonZeroU32,
    owner: Option<u32>,
    transaction_id: u32,
    transaction_line: Option<u32>,
}

#[derive(Default)]
pub(in crate::codebase::postgres::embedded::walk) struct State {
    objects: FxHashMap<NonZeroU32, Captured>,
    aliases: FxHashMap<NonZeroU32, Captured>,
    reads: RefCell<FxHashSet<u32>>,
    mutated: FxHashSet<NonZeroU32>,
}

fn owner(visitor: &ScopeVisitor<'_>) -> Option<u32> {
    visitor.member_query_proof.function_ids.last().copied()
}

impl Captured {
    fn apply(&self, fact: &mut EmbeddedSqlCall) {
        fact.transaction_id = Some(self.transaction_id);
        fact.transaction_line = self.transaction_line;
        fact.executor_transaction_known = true;
    }

    fn valid(&self, visitor: &ScopeVisitor<'_>) -> bool {
        !visitor.transaction_options.mutated.contains(&self.identity)
            && self.owner == owner(visitor)
            && visitor.scopes.iter().any(|scope| {
                scope
                    .get(&self.name)
                    .and_then(|binding| binding.value_identity)
                    == Some(self.identity)
            })
    }
}

pub(super) fn copy(source: &EmbeddedSqlCall, target: &mut EmbeddedSqlCall) {
    target.transaction_id = source.transaction_id;
    target.transaction_line = source.transaction_line;
    target.executor_transaction_known = source.executor_transaction_known;
}

fn captured(visitor: &ScopeVisitor<'_>, expression: &Expression<'_>) -> Option<Captured> {
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let identity = visitor.lookup(id.name.as_str())?.value_identity?;
    if let Some(captured) = visitor.transaction_options.aliases.get(&identity) {
        return captured.valid(visitor).then(|| captured.clone());
    }
    let mut fact = EmbeddedSqlCall::default();
    super::super::annotate_receiver(visitor, expression, id.span.start, &mut fact);
    (fact.executor_transaction_known && fact.transaction_id.is_some()).then(|| Captured {
        name: id.name.to_string(),
        identity,
        owner: owner(visitor),
        transaction_id: fact.transaction_id.unwrap(),
        transaction_line: fact.transaction_line,
    })
}

pub(super) fn alias(
    visitor: &ScopeVisitor<'_>,
    expression: &Expression<'_>,
    fact: &mut EmbeddedSqlCall,
) -> bool {
    if visitor.transaction_options.aliases.is_empty() {
        return false;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
        return false;
    };
    let Some(identity) = visitor
        .lookup(id.name.as_str())
        .and_then(|binding| binding.value_identity)
    else {
        return false;
    };
    let Some(captured) = visitor.transaction_options.aliases.get(&identity) else {
        return false;
    };
    if captured.valid(visitor) {
        captured.apply(fact);
    }
    true
}

pub(super) fn option(
    visitor: &ScopeVisitor<'_>,
    id: &IdentifierReference<'_>,
) -> Option<EmbeddedSqlCall> {
    if !visitor.executor_options.track_transaction_helpers {
        return None;
    }
    let identity = visitor.lookup(id.name.as_str())?.value_identity?;
    let captured = visitor.transaction_options.objects.get(&identity)?;
    visitor
        .transaction_options
        .reads
        .borrow_mut()
        .insert(id.span.start);
    captured.valid(visitor).then(|| {
        let mut fact = EmbeddedSqlCall::default();
        captured.apply(&mut fact);
        fact
    })
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn mutate_transaction_receiver(
        &mut self,
        expression: &Expression<'_>,
    ) {
        boundary::mutate(self, expression);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_transaction_options(
        &mut self,
        declaration: &VariableDeclarator<'_>,
    ) {
        if !self.executor_options.track_transaction_helpers
            || self.executor_options.executor_option_properties.is_empty()
            || !self.declaring_const
        {
            return;
        }
        let (BindingPattern::BindingIdentifier(name), Some(init)) =
            (&declaration.id, &declaration.init)
        else {
            return;
        };
        let Some(identity) = self
            .lookup(name.name.as_str())
            .and_then(|binding| binding.value_identity)
        else {
            return;
        };
        if matches!(unwrap_ts_wrappers(init), Expression::ObjectExpression(_)) {
            let Some(Some(value)) =
                receiver(init, &self.executor_options.executor_option_properties)
            else {
                return;
            };
            if let Some(captured) = captured(self, value) {
                self.transaction_options.objects.insert(identity, captured);
            }
        } else if let Some(value) = value_receiver(init) {
            if let Some(captured) = captured(self, value) {
                self.transaction_options.aliases.insert(identity, captured);
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn poison_transaction_options(
        &mut self,
        id: &IdentifierReference<'_>,
    ) {
        if self.transaction_options.objects.is_empty()
            || self
                .transaction_options
                .reads
                .borrow_mut()
                .remove(&id.span.start)
        {
            return;
        }
        if let Some(identity) = self
            .lookup(id.name.as_str())
            .and_then(|binding| binding.value_identity)
        {
            self.transaction_options.objects.remove(&identity);
        }
    }
}
