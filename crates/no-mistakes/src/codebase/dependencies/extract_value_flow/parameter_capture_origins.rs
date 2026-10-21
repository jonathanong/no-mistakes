use super::*;
type Key = (usize, String);

pub(in super::super) struct Reassignment {
    pub(in super::super) key: Key,
    pub(in super::super) value: ValueExpression,
    pub(in super::super) completed_at: u32,
    pub(in super::super) context: Option<CallableId>,
    pub(in super::super) certain: bool,
}
struct Captures {
    initial: Vec<ValueExpression>,
    events: Vec<Reassignment>,
    owner: CallableId,
    foreign: bool,
}
#[derive(Default)]
pub(in super::super) struct CaptureOrigins {
    bindings: FxHashMap<Key, Captures>,
    #[cfg(feature = "test-instrumentation")]
    comparisons: std::sync::atomic::AtomicUsize,
}
impl CaptureOrigins {
    pub(in super::super) fn new(
        collector: &ImportCollector,
        values: &FxHashMap<Key, Vec<ValueExpression>>,
    ) -> Self {
        let mut result = Self::default();
        if collector.receiver_escapes.reassigned.is_empty() {
            return result;
        }
        // Callback argument forwarding can add origins later. Keep its existing
        // union rather than claim a complete invocation snapshot here.
        let mut callbacks = fx_set();
        for call in &collector.value_flow.calls {
            for argument in call.arguments.iter() {
                if let Some(owner) = super::callable_owner(collector, argument) {
                    callbacks.insert(owner);
                }
            }
        }
        let mut owners = fx_map();
        for parameter in &collector.value_flow.parameters {
            if parameter.projection.is_empty() && !callbacks.contains(&parameter.owner) {
                owners
                    .entry((parameter.scope, parameter.name.clone()))
                    .or_insert(parameter.owner);
            }
        }
        for event in &collector.receiver_escapes.reassigned {
            let Some(owner) = owners.get(&event.key) else {
                continue;
            };
            let captures = result
                .bindings
                .entry(event.key.clone())
                .or_insert_with(|| Captures {
                    initial: values.get(&event.key).cloned().unwrap_or_default(),
                    events: Vec::new(),
                    owner: *owner,
                    foreign: false,
                });
            captures.foreign |= event.context != Some(*owner);
            captures.events.push(Reassignment {
                key: event.key.clone(),
                value: event.value.clone(),
                completed_at: event.completed_at,
                context: event.context,
                certain: event.certain,
            });
        }
        for (key, captures) in &mut result.bindings {
            captures.events.sort_by_key(|event| event.completed_at);
            let completions = captures
                .events
                .iter()
                .map(|event| event.completed_at)
                .collect();
            captures.foreign |= !collector.parameter_capture_writes_covered(key, &completions);
        }
        result
    }
    /// Only ordered writes in the same invocation select a capture generation.
    /// Unknown execution order retains the historical union at the caller.
    pub(super) fn at(&self, key: &Key, read: nested::RootRead) -> Option<&[ValueExpression]> {
        let captures = self.bindings.get(key)?;
        if captures.foreign || read.1 != Some(captures.owner) {
            return None;
        }
        let index = captures.events.partition_point(|event| {
            #[cfg(feature = "test-instrumentation")]
            self.comparisons
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            event.completed_at <= read.0
        });
        if index == 0 {
            return Some(captures.initial.as_slice());
        }
        let event = &captures.events[index - 1];
        (event.certain && matches!(event.value, ValueExpression::Named { .. }))
            .then_some(std::slice::from_ref(&event.value))
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "parameter_capture_origins_tests.rs"]
mod tests;
