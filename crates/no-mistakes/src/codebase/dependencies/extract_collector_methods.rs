fn visit_call_expression_with_imports(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    let require_callee = is_require_resolve_callee(&call.callee)
        .then_some("require.resolve")
        .or_else(|| is_require_callee(&call.callee).then_some("require"));
    if let Some(callee) = require_callee.filter(|_| !collector.local_binding_shadows("require")) {
        if collector.should_record_call(callee) {
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                syntactic_caller: collector.current_syntactic_caller(),
                callee: callee.to_string(),
                line: import_line_at(&collector.line_starts, call.span.start as usize),
                offset: call.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity: collector.call_target_identity(callee),
                callee_binding_scope: collector.callee_binding_scope(callee),
                static_arg: call.arguments.first().and_then(static_path_argument),
                static_cwd: None,
            });
        }
    }
    if is_require_resolve_callee(&call.callee) && !collector.local_binding_shadows("require") {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(
                    specifier,
                    ImportKind::RequireResolve,
                    call.span.start as usize,
                );
            }
        }
    } else if is_require_callee(&call.callee) && !collector.local_binding_shadows("require") {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(specifier, ImportKind::Require, call.span.start as usize);
            }
        }
    } else if let Some(callee) = simple_callee_name(&call.callee) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                syntactic_caller: collector.current_syntactic_caller(),
                static_cwd: static_process_cwd_arg(&callee, &call.arguments),
                callee,
                line: import_line_at(&collector.line_starts, call.span.start as usize),
                offset: call.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity,
                callee_binding_scope,
                static_arg: call.arguments.first().and_then(static_path_argument),
            });
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, call.span.start as usize),
            call.span.start,
            InvocationKind::Call,
        );
    }
}

fn visit_new_expression_with_imports(collector: &mut ImportCollector, new: &NewExpression<'_>) {
    if let Some(callee) = simple_callee_name(&new.callee) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                syntactic_caller: collector.current_syntactic_caller(),
                static_cwd: None,
                callee,
                line: import_line_at(&collector.line_starts, new.span.start as usize),
                offset: new.span.start,
                is_callback: false,
                invocation: InvocationKind::Construct,
                target_identity,
                callee_binding_scope,
                static_arg: new.arguments.first().and_then(static_path_argument),
            });
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, new.span.start as usize),
            new.span.start,
            InvocationKind::Construct,
        );
    }
}

impl ImportCollector {
    fn record_unknown_call(&mut self, line: u32, offset: u32, invocation: InvocationKind) {
        let caller = self.current_function();
        if caller.is_none() {
            self.has_unknown_top_level_call = true;
        }
        self.unknown_callers.push(caller.clone());
        self.unknown_calls.push(UnknownCall {
            caller,
            line,
            offset,
            invocation,
        });
    }
}
