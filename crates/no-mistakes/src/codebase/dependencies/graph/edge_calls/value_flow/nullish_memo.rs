use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub(super) type NullishMemoHandle = Rc<RefCell<NullishMemo>>;
type Owner = (std::path::PathBuf, CallableId);

#[derive(Default)]
pub(super) struct NullishMemo {
    pub(super) generation: usize,
    entries: FxHashMap<nullish_key::NullishKey, Arc<Receipt>>,
    guard_reads: nullish_guards::GuardReads,
    #[cfg(feature = "test-instrumentation")]
    guard_entries: usize,
}
struct Receipt {
    value: FlowValue,
    origins: default_origins::Origins,
    parameters: FxHashSet<Owner>,
    writers: FxHashSet<Owner>,
    replacement: bool,
    guards: nullish_guards::Dependencies,
}

impl ValueFlowResolution<'_, '_> {
    pub(super) fn record_nullish_guard(&self, name: &(std::path::PathBuf, usize, String)) {
        if let Some(memo) = &self.nullish_memo {
            memo.borrow_mut()
                .guard_reads
                .append(std::iter::once(name), self.cache);
        }
    }

    pub(super) fn mark_nullish_cycle(&self) {
        if let Some(memo) = &self.nullish_memo {
            memo.borrow_mut().generation += 1;
        }
    }

    pub(super) fn memoized_nullish_binding(
        &mut self,
        path: &std::path::Path,
        binding: &ValueBinding,
        read: nullish_key::BindingRead,
        projection: Option<&member_projection::MemberProjection<'_>>,
    ) -> FlowValue {
        let Some(memo) = self.nullish_memo.clone() else {
            return self.value_projected(path, &binding.value, projection);
        };
        // Mutable factory bindings also use this catalog; their acquisitions
        // keep the existing uncached path. Other recorded bindings are immutable.
        if matches!(
            binding.value,
            ValueExpression::Call(..) | ValueExpression::Awaited(_)
        ) {
            return self.value_projected(path, &binding.value, projection);
        }
        let key = self.nullish_key(path, binding, read, projection);
        let cached = memo.borrow().entries.get(&key).cloned();
        if let Some(cached) = cached
            .filter(|receipt| memo.borrow().guard_reads.matches(&receipt.guards, &self.visiting, self.cache))
        {
            // Replay negative checks too: an outer cached computation must own
            // every ancestry assumption made by this shared receipt.
            memo.borrow_mut().guard_reads.append_ids(
                cached.guards.iter().map(|(id, _)| *id),
                self.cache,
            );
            #[cfg(feature = "test-instrumentation")]
            self.cache.record_nullish_work(1, 1);
            default_origins::merge(
                &mut self.default_origins,
                cached.origins.clone(),
                self.cache,
            );
            self.used_parameters
                .extend(cached.parameters.iter().cloned());
            self.writer_dependencies
                .borrow_mut()
                .extend(cached.writers.iter().cloned());
            self.replacement_read |= cached.replacement;
            return cached.value.clone();
        }
        // Named reads already hold their lexical guard. A normalized child
        // has a distinct frame key, or ParameterCache rejects its active lease.
        #[cfg(feature = "test-instrumentation")]
        self.cache.record_nullish_work(0, 1);
        let generation = memo.borrow().generation;
        let guard_start = memo.borrow().guard_reads.len();
        #[cfg(feature = "test-instrumentation")]
        self.cache.record_nullish_work(6, self.visiting.len());
        let entry_guards = self.visiting.clone();
        let origins = self.default_origins.take();
        let parameters = std::mem::take(&mut self.used_parameters);
        let writers = std::mem::take(self.writer_dependencies.get_mut());
        let replacement = std::mem::replace(&mut self.replacement_read, false);
        let value = self.value_projected(path, &binding.value, projection);
        let guards = memo
            .borrow()
            .guard_reads
            .capture(guard_start, &entry_guards, self.cache);
        let receipt = Arc::new(Receipt {
            value: value.clone(),
            origins: self.default_origins.take(),
            parameters: std::mem::replace(&mut self.used_parameters, parameters),
            writers: std::mem::replace(self.writer_dependencies.get_mut(), writers),
            replacement: self.replacement_read,
            guards,
        });
        self.default_origins = origins;
        default_origins::merge(
            &mut self.default_origins,
            receipt.origins.clone(),
            self.cache,
        );
        self.used_parameters
            .extend(receipt.parameters.iter().cloned());
        self.writer_dependencies
            .get_mut()
            .extend(receipt.writers.iter().cloned());
        self.replacement_read |= replacement;
        let stable = key == self.nullish_key(path, binding, read, projection);
        let mut state = memo.borrow_mut();
        if stable && state.generation == generation {
            #[cfg(feature = "test-instrumentation")]
            {
                state.guard_entries += receipt.guards.len();
            }
            let replaced = state.entries.insert(key, receipt);
            #[cfg(feature = "test-instrumentation")]
            {
                state.guard_entries -= replaced.as_ref().map_or(0, |receipt| receipt.guards.len());
                self.cache.record_nullish_peak(9, state.guard_entries);
            }
            #[cfg(not(feature = "test-instrumentation"))]
            let _ = replaced;
        }
        value
    }
}
