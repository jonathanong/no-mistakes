/// A syntactic per-item invocation. Resolution and reachability belong to the
/// prepared canonical graph, not this parser-owned occurrence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerItemCall {
    pub offset: u32,
    pub line: u32,
    pub callee: String,
    pub caller_id: Option<CallableId>,
    pub construct: String,
    pub is_callback: bool,
}

struct PerItemRange {
    span: oxc_span::Span,
    construct: String,
}

impl ImportCollector {
    fn record_per_item_range(&mut self, span: oxc_span::Span, caller_id: Option<CallableId>, construct: &str) {
        if self.collect_per_item_calls {
            self.per_item_ranges.entry(caller_id).or_default().push(PerItemRange { span, construct: construct.to_string() });
        }
    }

    fn record_per_item_invocations(&mut self, before: usize) {
        if !self.collect_per_item_calls { return; }
        for call in &self.function_calls[before..] {
            if call.is_callback { continue; }
            if let Some(range) = self.per_item_ranges.get(&call.caller_id).into_iter().flatten().rev().find(|range|
                range.span.start <= call.offset && call.offset < range.span.end
            ) {
                self.per_item_calls.push(PerItemCall { offset: call.offset, line: call.line,
                    callee: call.callee.clone(), caller_id: call.caller_id,
                    construct: range.construct.clone(), is_callback: false });
            }
        }
    }

    fn record_iteration_callback(&mut self, call: &CallExpression<'_>) -> Option<(u32, String)> {
        let callee = simple_callee_name(&call.callee)?;
        let (_, method) = callee.rsplit_once('.')?;
        if !matches!(method, "map" | "flatMap" | "forEach" | "filter" | "some" | "every" | "reduce" | "find") { return None; }
        let callback = call.arguments.first().and_then(Argument::as_expression)?;
        let construct = format!("{method} callback");
        match callback {
            Expression::ArrowFunctionExpression(function) => self.record_per_item_range(function.span, Some(CallableId(function.span.start)), &construct),
            Expression::FunctionExpression(function) => self.record_per_item_range(function.span, Some(CallableId(function.span.start)), &construct),
            _ => {
                let name = simple_callee_name(callback)?;
                // Repository/imported callback transitions belong in the same
                // canonical call collection; arbitrary values remain unknown.
                let identity = self.call_target_identity(&name);
                {
                    // Existing argument transitions already record repository functions.
                    // Imported/global/unknown named iteration callbacks must have the
                    // same graph identity regardless of optional per-item demand.
                    if identity != CallTargetIdentity::RepositoryFunction {
                        self.function_calls.push(FunctionCall { caller: self.current_function(), caller_id: self.current_function_id(), syntactic_caller: self.current_syntactic_caller(), callee: name.clone(), line: 0, offset: callback.span().start, is_callback: true, invocation: InvocationKind::Callback, target_identity: identity, callee_binding_scope: self.callee_binding_scope(&name), static_arg: None, static_cwd: None });
                    }
                    if self.collect_per_item_calls { self.per_item_calls.push(PerItemCall { offset: callback.span().start,
                        line: import_line_at(&self.line_starts, callback.span().start as usize),
                        callee: name.clone(), caller_id: self.current_function_id(), construct, is_callback: true }); }
                }
                return Some((callback.span().start, name));
            }
        }
        None
    }
}

fn visit_call_expression_per_item<'a>(collector: &mut ImportCollector, call: &CallExpression<'a>) {
    let iteration = collector.record_iteration_callback(call);
    let before = collector.function_calls.len();
    visit_call_expression_with_imports(collector, call);
    if let Some((offset, name)) = iteration {
        // Generic repository argument transitions predate precise callback
        // positions. Preserve this actual use offset for canonical alias/TDZ
        // resolution without adding a competing binding resolver.
        for transition in &mut collector.function_calls[before..] {
            if transition.is_callback && transition.callee == name { transition.offset = offset; }
        }
    }
    collector.record_per_item_invocations(before);
    walk::walk_call_expression(collector, call);
}
