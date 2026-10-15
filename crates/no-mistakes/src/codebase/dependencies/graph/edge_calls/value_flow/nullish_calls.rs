use super::*;
use std::sync::Arc;

impl ValueFlowResolution<'_, '_> {
    // Receipts stay with their syntax branches until this canonical projection.
    pub(super) fn nullish_call_value(
        &self,
        value: FlowValue,
        site: &mut ResolvedCallSite,
    ) -> FlowValue {
        let root = value;
        let mut pending = vec![&root];
        let mut seen = fx_set();
        let mut known = Vec::new();
        let mut unknown = false;
        while let Some(value) = pending.pop() {
            if !seen.insert(value as *const FlowValue) {
                continue;
            }
            #[cfg(feature = "test-instrumentation")]
            self.cache.record_nullish_work(2, 1);
            match value {
                FlowValue::Nullish(branches) => {
                    pending.push(branches.fallback.as_ref());
                    pending.push(branches.value.as_ref());
                }
                FlowValue::Annotated {
                    executor,
                    file,
                    source,
                } => {
                    if callable_receipt(executor) {
                        if let Some(origin) = self.annotation_origin(file, source) {
                            site.typed_from.push(origin);
                        }
                    }
                    pending.push(executor.as_ref());
                }
                FlowValue::Provenance {
                    executor,
                    typed,
                    members,
                    ..
                } => {
                    if callable_receipt(executor) {
                        site.typed_from.extend(typed.iter().cloned());
                        site.returned_member_from.extend(members.iter().cloned());
                    }
                    pending.push(executor.as_ref());
                }
                FlowValue::Returned {
                    factory, executor, ..
                } => {
                    site.returned_from.extend(factory.iter().cloned());
                    pending.push(executor.as_ref());
                }
                FlowValue::Bound { target, .. } => pending.push(target.as_ref()),
                FlowValue::Target(ResolvedCallTarget::Alternatives(alternatives)) => {
                    known.extend_from_slice(alternatives.known());
                    unknown |= alternatives.has_unknown;
                }
                leaf => match replacement_values::replacement_candidate(self, leaf.clone()) {
                    Some(candidate) => known.push(candidate),
                    None => unknown = true,
                },
            }
        }
        site.typed_from.sort_unstable();
        site.typed_from.dedup();
        site.returned_from.sort_unstable();
        site.returned_from.dedup();
        site.returned_member_from.sort_unstable();
        site.returned_member_from.dedup();
        if known.is_empty() {
            return FlowValue::Unknown;
        }
        FlowValue::Target(ResolvedCallTarget::Alternatives(Arc::new(
            ResolvedCallAlternatives::new(known, unknown),
        )))
    }
}

fn callable_receipt(value: &FlowValue) -> bool {
    !matches!(
        value.executor(),
        FlowValue::Promise(_)
            | FlowValue::Uncallable
            | FlowValue::Undefined
            | FlowValue::Object { .. }
            | FlowValue::Array { .. }
    )
}
