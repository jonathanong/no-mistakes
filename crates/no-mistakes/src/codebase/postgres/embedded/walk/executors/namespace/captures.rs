mod lookup;
use super::super::ScopeVisitor;
use super::identity::Origin;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

#[derive(Clone)]
pub(super) struct QueryCapture {
    pub(super) key: (u32, String),
    pub(super) origins: Vec<Origin>,
    pub(super) owner: u32,
    pub(super) event: u32,
    pub(super) trusted: bool,
}

pub(super) struct CapturedCall {
    pub(super) key: (u32, String),
    pub(super) span: (u32, u32),
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn note_captured_call(
        &mut self,
        call: &CallExpression<'_>,
        callee: &Expression<'_>,
        uncertain: bool,
    ) {
        let Expression::Identifier(name) = unwrap_ts_wrappers(callee) else {
            return;
        };
        if uncertain {
            self.invalidate_query_capture(name.name.as_str());
        }
        let Some(key) = self.capture_key(name.name.as_str()) else {
            return;
        };
        self.member_query_proof.captured_calls.push(CapturedCall {
            key,
            span: (call.span.start, call.span.end),
        });
    }

    pub(in crate::codebase::postgres::embedded::walk) fn capture_scope(
        &self,
        name: &str,
    ) -> Option<u32> {
        let (index, _) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))?;
        Some(self.member_query_proof.scope_ids[index])
    }

    fn capture_key(&self, name: &str) -> Option<(u32, String)> {
        Some((self.capture_scope(name)?, name.to_string()))
    }

    fn query_capture(&self, expression: &Expression<'_>) -> Option<&QueryCapture> {
        let Expression::Identifier(name) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        self.named_query_capture(name.name.as_str())
    }

    pub(in crate::codebase::postgres::embedded::walk) fn has_query_capture(
        &self,
        expression: &Expression<'_>,
    ) -> bool {
        self.query_capture(expression).is_some()
    }

    pub(in crate::codebase::postgres::embedded::walk) fn query_capture_origins(
        &self,
        expression: &Expression<'_>,
    ) -> Option<Vec<Origin>> {
        self.query_capture(expression)
            .map(|capture| capture.origins.clone())
    }

    pub(in crate::codebase::postgres::embedded::walk) fn invalidate_query_capture(
        &mut self,
        name: &str,
    ) {
        self.invalidate_capture(name, false);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn escape_query_capture(
        &mut self,
        name: &str,
    ) {
        self.invalidate_capture(name, true);
    }

    fn invalidate_capture(&mut self, name: &str, escaped: bool) {
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let Some(scope) = self.capture_scope(name) else {
            return;
        };
        let proof = &mut self.member_query_proof;
        if let Some(indices) = proof
            .capture_indices
            .get(&scope)
            .and_then(|names| names.get(name))
        {
            for &index in indices {
                let capture = &mut proof.captures[index];
                if !escaped || owner == 0 || capture.owner != 0 {
                    capture.trusted = false;
                }
            }
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn copy_query_capture(
        &mut self,
        name: &str,
        source: &Expression<'_>,
        trusted: bool,
    ) -> bool {
        let Some(mut capture) = self.query_capture(source).cloned() else {
            return false;
        };
        let Some(key) = self.capture_key(name) else {
            return false;
        };
        capture.key = key;
        capture.trusted &= trusted;
        // Preserve the original lookup event (docs/rules/postgres-no-test-state-mutation.md).
        self.store_query_capture(capture);
        true
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_query_capture(
        &mut self,
        name: &str,
        receiver: &Expression<'_>,
    ) {
        self.note_query_capture_with_certainty(name, receiver, true);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_query_capture_with_certainty(
        &mut self,
        name: &str,
        receiver: &Expression<'_>,
        trusted: bool,
    ) {
        let Some(key) = self.capture_key(name) else {
            return;
        };
        let owner = self
            .member_query_proof
            .function_ids
            .last()
            .copied()
            .unwrap_or(0);
        let origins = self.receiver_origins(receiver);
        let event = self.member_query_proof.next_event;
        self.store_query_capture(QueryCapture {
            key,
            origins,
            owner,
            event,
            trusted,
        });
    }

    fn store_query_capture(&mut self, capture: QueryCapture) {
        if let Some(binding) = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(&capture.key.1))
        {
            binding.executor_alias = Some(capture.trusted);
        }
        self.member_query_proof
            .capture_indices
            .entry(capture.key.0)
            .or_default()
            .entry(capture.key.1.clone())
            .or_default()
            .push(self.member_query_proof.captures.len());
        self.member_query_proof.captures.push(capture);
    }
}
