use crate::codebase::dependencies::extract::{
    ValueAssignedMutations, ValueExpression, ValueMutationHistory,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct WriteIndex {
    writes: FxHashSet<String>,
    namespace: FxHashMap<String, std::collections::BTreeSet<String>>,
    #[cfg(feature = "test-instrumentation")]
    pub(super) work: usize,
}
impl WriteIndex {
    pub(super) fn new(ts: Option<&TsFileFacts>) -> Self {
        let mut result = Self::default();
        let Some(ts) = ts else {
            return result;
        };
        for by_name in ts
            .value_flow
            .receiver_mutations
            .iter()
            .flat_map(|histories| histories.values())
        {
            for (local, history) in by_name {
                result.record();
                let mut written =
                    named_members(history.assigned.as_deref()) || history.has_unassigned;
                for (member, child) in &history.members {
                    result.record();
                    let nested = nested_write(child);
                    written |= nested;
                    if history.has_unassigned || nested {
                        result
                            .namespace
                            .entry(local.clone())
                            .or_default()
                            .insert(member.clone());
                    }
                }
                if written {
                    result.writes.insert(local.clone());
                }
            }
        }
        for call in &ts.value_flow.calls {
            result.record();
            if is_property_mutator(&call.callee) {
                if let Some(ValueExpression::Named { name, .. }) = call.arguments.first() {
                    result.writes.insert(name.clone());
                }
            }
        }
        result
    }
    pub(super) fn member_write(&self, local: &str) -> bool {
        self.writes.contains(local)
    }
    pub(super) fn namespace_writes(&self, local: &str) -> Vec<String> {
        self.namespace
            .get(local)
            .map_or_else(Vec::new, |names| names.iter().cloned().collect())
    }
    fn record(&mut self) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.work += 1;
        }
    }
}
fn is_property_mutator(callee: &ValueExpression) -> bool {
    let ValueExpression::Member(base, method) = callee else {
        return false;
    };
    let ValueExpression::Named { name, .. } = base.as_ref() else {
        return false;
    };
    matches!(
        (name.as_str(), method.as_str()),
        (
            "Object",
            "assign" | "defineProperty" | "defineProperties" | "setPrototypeOf"
        ) | (
            "Reflect",
            "defineProperty" | "deleteProperty" | "set" | "setPrototypeOf"
        )
    )
}
fn nested_write(history: &ValueMutationHistory) -> bool {
    named_members(history.assigned.as_deref()) || history.has_unassigned
}
fn named_members(assigned: Option<&ValueAssignedMutations>) -> bool {
    assigned.is_some_and(|assigned| !assigned.members.is_empty() || assigned.dynamic.is_some())
}
