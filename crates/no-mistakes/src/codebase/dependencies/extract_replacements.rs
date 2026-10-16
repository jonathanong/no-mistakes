#[derive(Clone, Copy)]
struct ReplacementRegion {
    start: u32,
    end: u32,
    owner: Option<CallableId>,
    repeated: bool,
    may_repeat: bool,
    index: Option<usize>,
}

impl ImportCollector {
    fn replacement_unconditional(&self) -> bool {
        let owner = self.current_function_id();
        if let Some(region) = self
            .replacement_region
            .filter(|region| region.owner == owner)
        {
            return region.repeated;
        }
        !self.conditional_execution
            && !self
                .function_id_stack
                .last()
                .map_or(self.replacement_root_unsafe, |(_, invalid)| *invalid)
    }
    fn replacement_region_index(&mut self) -> Option<usize> {
        let owner = self.current_function_id();
        let region = self
            .replacement_region
            .as_mut()
            .filter(|region| region.owner == owner)?;
        if let Some(index) = region.index {
            return Some(index);
        }
        let facts = self
            .value_flow
            .replacements
            .as_mut()
            .and_then(std::sync::Arc::get_mut)
            .filter(|facts| facts.requires_receiver_proof())?;
        let index = facts.regions.len();
        facts.regions.push(ValueRepeatRegion {
            start: region.start,
            end: region.end,
            owner,
            repeated: false,
            may_repeat: region.may_repeat,
        });
        region.index = Some(index);
        Some(index)
    }
    fn record_replacement_call(&mut self, call: &CallExpression<'_>) {
        if !self.collect_value_flow {
            return;
        }
        // Unknown nested execution, including recursion, is outside the closed
        // straightline owner proof. This needs no extra binding lookup.
        if self.current_function_id().is_some()
            && matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee),
                Expression::Identifier(_)
            )
        {
            self.invalidate_replacement_execution();
        }
        if !self
            .value_flow
            .replacements
            .as_ref()
            .is_some_and(|facts| facts.needs_call_proofs)
        {
            return;
        }
        let region = self.replacement_region_index();
        let offset = call.span.start;
        let call = ValueRepeatCall {
            owner: self.current_function_id(),
            region,
            unconditional: call.arguments.is_empty()
                && self.replacement_unconditional()
                && !call_has_execution_guard(call)
                && self.replacement_statement_call == Some((call.span.start, call.span.end)),
        };
        self.value_flow
            .replacements
            .as_mut()
            .and_then(std::sync::Arc::get_mut)
            .expect("write facts")
            .calls
            .insert(offset, call);
    }
    // Typed suspension/delete hooks avoid replacement work on ordinary expressions.
    fn invalidate_replacement_expression(&mut self) {
        #[cfg(feature = "test-instrumentation")]
        {
            self.value_flow.replacement_expression_guard_entries += 1;
        }
        if self.collect_value_flow {
            #[cfg(feature = "test-instrumentation")]
            {
                self.value_flow.replacement_expression_checks += 1;
            }
            self.invalidate_replacement_execution();
        }
    }
    fn invalidate_replacement_execution(&mut self) {
        if !self.collect_value_flow {
            return;
        }
        let owner = self.current_function_id();
        if let Some((_, invalid)) = self.function_id_stack.last_mut() {
            *invalid = true;
        } else {
            self.replacement_root_unsafe = true;
        }
        if let Some(region) = self
            .replacement_region
            .as_mut()
            .filter(|region| region.owner == owner)
        {
            region.repeated = false;
        }
        if let (Some(owner), Some(facts)) = (
            owner,
            self.value_flow
                .replacements
                .as_mut()
                .and_then(std::sync::Arc::get_mut),
        ) {
            if facts.requires_receiver_proof() {
                facts.unsafe_owners.insert(owner);
            }
        }
    }
}

fn with_replacement_region(
    collector: &mut ImportCollector,
    span: oxc_span::Span,
    repeated: bool,
    may_repeat: bool,
    visit: impl FnOnce(&mut ImportCollector),
) {
    if !collector.collect_value_flow {
        visit(collector);
        return;
    }
    #[cfg(feature = "test-instrumentation")]
    {
        collector.value_flow.replacement_region_frames += 1;
    }
    let owner = collector.current_function_id();
    // Nested control is unsupported, rather than replayed as an interpreter.
    if collector
        .replacement_region
        .is_some_and(|region| region.owner == owner)
    {
        collector.invalidate_replacement_execution();
    }
    let previous = collector.replacement_region.replace(ReplacementRegion {
        start: span.start,
        end: span.end,
        owner,
        repeated,
        may_repeat,
        index: None,
    });
    visit(collector);
    // Retain an enclosing region even when its writes were recorded in a
    // nested/unsupported region. Reads there must not reuse a stale original.
    if collector.value_flow.has_receiver_replacement_facts() {
        collector.replacement_region_index();
    }
    let region = collector
        .replacement_region
        .take()
        .expect("active repeat region");
    if let Some(index) = region.index {
        collector
            .value_flow
            .replacements
            .as_mut()
            .and_then(std::sync::Arc::get_mut)
            .expect("region facts")
            .regions[index]
            .repeated = region.repeated;
    }
    collector.replacement_region = previous;
}

include!("extract_replacement_targets.rs");
