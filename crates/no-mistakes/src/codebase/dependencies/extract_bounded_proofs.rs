impl BoundedDomains {
    fn guard_valid(&self, dependencies: &[(usize, String)]) -> bool {
        dependencies.iter().all(|(scope, name)| {
            self.values
                .get(scope)
                .and_then(|values| values.get(name))
                .is_some_and(|value| !matches!(value, BoundedValue::Unknown))
        })
    }
}

impl ImportCollector {
    fn inherited_bounded_dependencies(&self, offset: u32) -> Vec<(usize, String)> {
        let mut dependencies = self
            .per_item_ranges
            .get(&self.current_function_id())
            .into_iter()
            .flatten()
            .filter(|range| range.span.start <= offset && offset < range.span.end)
            .filter_map(|range| range.guard.as_ref())
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        dependencies.dedup();
        dependencies
    }

    fn record_closed_per_item_range(
        &mut self,
        span: oxc_span::Span,
        caller_id: Option<CallableId>,
        construct: &str,
        mut guard: BoundedGuard,
    ) {
        guard.extend(self.inherited_bounded_dependencies(span.start));
        guard.sort_unstable();
        guard.dedup();
        self.per_item_ranges
            .entry(caller_id)
            .or_default()
            .push(PerItemRange {
                span,
                construct: construct.to_string(),
                guard: Some(guard),
            });
    }

    fn record_bounded_batch_call(&mut self, offset: u32) {
        let dependencies = self.inherited_bounded_dependencies(offset);
        self.bounded_batch_calls.insert(offset, dependencies);
    }

    fn finish_bounded_per_item_calls(&mut self) {
        if !self.collect_per_item_calls {
            return;
        }
        let domains = &self.bounded_domains;
        let ranges = &self.per_item_ranges;
        let batch_calls = &self.bounded_batch_calls;
        self.per_item_calls.retain_mut(|call| {
            if call.is_callback {
                return true;
            }
            if batch_calls
                .get(&call.offset)
                .is_some_and(|guard| domains.guard_valid(guard))
            {
                return false;
            }
            let active = ranges
                .get(&call.caller_id)
                .into_iter()
                .flatten()
                .rev()
                .find(|range| {
                    range.span.start <= call.offset
                        && call.offset < range.span.end
                        && range
                            .guard
                            .as_ref()
                            .is_none_or(|guard| !domains.guard_valid(guard))
                });
            if let Some(range) = active {
                call.construct.clone_from(&range.construct);
                true
            } else {
                false
            }
        });
    }
}
