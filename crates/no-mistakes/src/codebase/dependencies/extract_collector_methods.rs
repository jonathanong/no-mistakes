fn visit_call_expression_with_imports(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    let require_callee = is_require_resolve_callee(&call.callee)
        .then_some("require.resolve")
        .or_else(|| is_require_callee(&call.callee).then_some("require"));
    if let Some(callee) = require_callee.filter(|_| !collector.local_binding_shadows("require")) {
        if collector.should_record_call(callee) {
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                caller_id: collector.current_function_id(),
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
            if is_static_getter(collector, &callee) {
                collector.record_unknown_call(
                    import_line_at(&collector.line_starts, call.span.start as usize),
                    call.span.start,
                    InvocationKind::Call,
                );
                record_callable_argument_transitions(collector, call);
                return;
            }
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                caller_id: collector.current_function_id(),
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
            if has_dynamic_static_member_receiver(&call.callee) {
                collector.record_unknown_call(
                    import_line_at(&collector.line_starts, call.span.start as usize),
                    call.span.start,
                    InvocationKind::Call,
                );
            }
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, call.span.start as usize),
            call.span.start,
            InvocationKind::Call,
        );
    }
    record_callable_argument_transitions(collector, call);
}

fn record_callable_argument_transitions(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    for argument in &call.arguments {
        let Some(expression) = argument.as_expression() else {
            continue;
        };
        let Some(callee) = simple_callee_name(expression) else {
            continue;
        };
        if collector.call_target_identity(&callee) != CallTargetIdentity::RepositoryFunction {
            continue;
        }
        collector.function_calls.push(FunctionCall {
            caller: collector.current_function(),
            caller_id: collector.current_function_id(),
            syntactic_caller: collector.current_syntactic_caller(),
            callee_binding_scope: collector.callee_binding_scope(&callee),
            callee,
            line: 0,
            offset: 0,
            is_callback: true,
            invocation: InvocationKind::Callback,
            target_identity: CallTargetIdentity::RepositoryFunction,
            static_arg: None,
            static_cwd: None,
        });
    }
}

fn visit_new_expression_with_imports(collector: &mut ImportCollector, new: &NewExpression<'_>) {
    if let Some(callee) = simple_callee_name(&new.callee) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                caller_id: collector.current_function_id(),
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
            if has_dynamic_static_member_receiver(&new.callee) {
                collector.record_unknown_call(
                    import_line_at(&collector.line_starts, new.span.start as usize),
                    new.span.start,
                    InvocationKind::Construct,
                );
            }
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, new.span.start as usize),
            new.span.start,
            InvocationKind::Construct,
        );
    }
}

fn visit_tagged_template_expression_with_imports(
    collector: &mut ImportCollector,
    tagged: &TaggedTemplateExpression<'_>,
) {
    if let Some(callee) = simple_callee_name(&tagged.tag) {
        if collector.should_record_call(&callee) {
            let target_identity = collector.call_target_identity(&callee);
            let callee_binding_scope = collector.callee_binding_scope(&callee);
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                caller_id: collector.current_function_id(),
                syntactic_caller: collector.current_syntactic_caller(),
                static_cwd: None,
                callee,
                line: import_line_at(&collector.line_starts, tagged.span.start as usize),
                offset: tagged.span.start,
                is_callback: false,
                invocation: InvocationKind::Call,
                target_identity,
                callee_binding_scope,
                static_arg: None,
            });
            if has_dynamic_static_member_receiver(&tagged.tag) {
                collector.record_unknown_call(
                    import_line_at(&collector.line_starts, tagged.quasi.span.start as usize),
                    tagged.quasi.span.start,
                    InvocationKind::Call,
                );
            }
        }
    } else {
        collector.record_unknown_call(
            import_line_at(&collector.line_starts, tagged.quasi.span.start as usize),
            tagged.quasi.span.start,
            InvocationKind::Call,
        );
    }
}

impl ImportCollector {
    fn record_unknown_call(&mut self, line: u32, offset: u32, invocation: InvocationKind) {
        let caller = self.current_function();
        if caller.is_none() {
            self.has_unknown_top_level_call = true;
        }
        self.unknown_calls.push(UnknownCall {
            caller,
            caller_id: self.current_function_id(),
            line,
            offset,
            invocation,
        });
    }
}
