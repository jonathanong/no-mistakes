use super::*;
type Key = (usize, String);
type Selection<'a> = (&'a [ValueExpression], bool, Option<&'a [ValueExpression]>);
#[path = "parameter_capture_conditionals.rs"]
mod conditionals;

pub(in super::super) struct Reassignment {
    pub(in super::super) key: Key,
    pub(in super::super) value: ValueExpression,
    pub(in super::super) completed_at: u32,
    pub(in super::super) context: Option<CallableId>,
    pub(in super::super) certain: bool,
    pub(in super::super) recovered: bool,
}
struct Captures {
    initial: Vec<ValueExpression>,
    possible_initial: Vec<ValueExpression>,
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
        let possible = conditionals::initial(collector);
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
            let captures = result.bindings.entry(event.key.clone()).or_insert_with(|| {
                let (possible_initial, initial) = values
                    .get(&event.key)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .partition(|value| {
                        conditionals::offset(value).is_some_and(|offset| possible.contains(&offset))
                    });
                Captures {
                    initial,
                    possible_initial,
                    events: Vec::new(),
                    owner: *owner,
                    foreign: false,
                }
            });
            captures.foreign |= event.context != Some(*owner);
            captures.events.push(Reassignment {
                key: event.key.clone(),
                value: event.value.clone(),
                completed_at: event.completed_at,
                context: event.context,
                certain: event.certain,
                recovered: event.recovered,
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

    /// Add only origins discovered by forwarding after reassignment. Raw
    /// reassignment values are already in `values` before this call and must
    /// remain separate from the initial capture generation.
    pub(in super::super) fn include_late_forwarded(
        &mut self,
        collector: &ImportCollector,
        slots: &FxHashMap<CallableId, FxHashMap<usize, Key>>,
        values: &mut FxHashMap<Key, Vec<ValueExpression>>,
        forward: impl FnOnce(&mut FxHashMap<Key, Vec<ValueExpression>>),
    ) {
        let starts = self
            .bindings
            .keys()
            .map(|key| (key.clone(), values.get(key).map_or(0, Vec::len)))
            .collect::<Vec<_>>();
        forward(values);
        if starts.is_empty() {
            return;
        }
        let conditional = conditionals::keys(collector, slots, values);
        for (key, start) in starts {
            let Some(additions) = values.get(&key).and_then(|values| values.get(start..)) else {
                continue;
            };
            let captures = self.bindings.get_mut(&key).expect("captured key");
            for value in additions {
                if captures.initial.contains(value) {
                    continue;
                }
                let initial = if conditional.contains(&key)
                    || conditionals::key(value).is_some_and(|key| conditional.contains(&key))
                {
                    &mut captures.possible_initial
                } else {
                    &mut captures.initial
                };
                if !initial.contains(value) {
                    initial.push(value.clone());
                }
            }
        }
    }

    /// Only ordered writes in the same invocation select a capture generation.
    /// Unknown execution order retains the historical union at the caller.
    pub(super) fn at(&self, key: &Key, read: nested::RootRead) -> Option<Selection<'_>> {
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
            // Forwarding through a reassigned callback can populate the legacy
            // union after this snapshot. Missing initial facts prove no absence.
            return (!captures.initial.is_empty() || !captures.possible_initial.is_empty())
                .then_some((
                    captures.initial.as_slice(),
                    false,
                    (!captures.possible_initial.is_empty())
                        .then_some(captures.possible_initial.as_slice()),
                ));
        }
        let event = &captures.events[index - 1];
        (event.certain && matches!(event.value, ValueExpression::Named { .. })).then_some((
            std::slice::from_ref(&event.value),
            event.recovered,
            None,
        ))
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "parameter_capture_origins_tests.rs"]
mod tests;
