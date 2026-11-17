use super::{EmbeddedSqlCall, ScopeVisitor};
use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Expression, ObjectPropertyKind, VariableDeclarator};
use oxc_span::GetSpan;
use std::num::NonZeroU32;

#[derive(Default)]
pub(super) struct State {
    aliases: FxHashMap<u32, FxHashSet<String>>,
    fields: FxHashMap<NonZeroU32, FxHashSet<String>>,
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn is_transaction_candidate(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
            return false;
        };
        self.executor_options.track_transaction_helpers
            && self.capture_scope(id.name.as_str()).is_some_and(|scope| {
                self.transaction_options
                    .candidates
                    .aliases
                    .get(&scope)
                    .is_some_and(|names| names.contains(id.name.as_str()))
            })
    }

    fn candidate_value(&self, expression: &Expression<'_>) -> bool {
        if self.is_transaction_candidate(expression)
            || (self.proven_executor(expression) && !self.namespace_expression(expression))
        {
            return true;
        }
        if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expression) {
            return self
                .candidate_fields(&member.object)
                .is_some_and(|fields| fields.contains(member.property.name.as_str()));
        }
        let mut fact = EmbeddedSqlCall::default();
        super::super::annotate_receiver(self, expression, expression.span().start, &mut fact);
        fact.transaction_id.is_some()
    }

    fn candidate_fields(&self, expression: &Expression<'_>) -> Option<&FxHashSet<String>> {
        let Expression::Identifier(id) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        let identity = self.lookup(id.name.as_str())?.value_identity?;
        self.transaction_options.candidates.fields.get(&identity)
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_transaction_candidate(
        &mut self,
        name: &str,
        expression: &Expression<'_>,
    ) {
        if self.executor_options.track_transaction_helpers && self.candidate_value(expression) {
            self.mark_transaction_candidate(name);
        }
    }

    fn mark_transaction_candidate(&mut self, name: &str) {
        self.capture_scope(name).into_iter().for_each(|scope| {
            self.transaction_options
                .candidates
                .aliases
                .entry(scope)
                .or_default()
                .insert(name.to_string());
        });
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_transaction_candidates(
        &mut self,
        declaration: &VariableDeclarator<'_>,
    ) {
        let Some(init) = &declaration.init else {
            return;
        };
        match &declaration.id {
            BindingPattern::BindingIdentifier(name) => {
                self.note_transaction_candidate(name.name.as_str(), init);
                let Expression::ObjectExpression(object) = unwrap_ts_wrappers(init) else {
                    return;
                };
                let fields = object
                    .properties
                    .iter()
                    .filter_map(|property| {
                        let ObjectPropertyKind::ObjectProperty(property) = property else {
                            return None;
                        };
                        self.candidate_value(&property.value)
                            .then(|| static_property_key_name(&property.key))
                            .flatten()
                            .map(|key| key.to_string())
                    })
                    .collect::<FxHashSet<_>>();
                if fields.is_empty() {
                    return;
                }
                if let Some(identity) = self
                    .lookup(name.name.as_str())
                    .and_then(|binding| binding.value_identity)
                {
                    self.transaction_options
                        .candidates
                        .fields
                        .insert(identity, fields);
                }
            }
            BindingPattern::ObjectPattern(object) => {
                let fields = self.candidate_fields(init).cloned().unwrap_or_default();
                for property in &object.properties {
                    let Some(key) = static_property_key_name(&property.key) else {
                        continue;
                    };
                    let BindingPattern::BindingIdentifier(name) = &property.value else {
                        continue;
                    };
                    if fields.contains(key.as_ref()) {
                        self.mark_transaction_candidate(name.name.as_str());
                    }
                }
            }
            _ => {}
        }
    }
}
