fn visit_call_expression_with_imports(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    if is_require_resolve_callee(&call.callee) {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(specifier, ImportKind::RequireResolve, call.span.start as usize);
            }
        }
    } else if is_require_callee(&call.callee) {
        if let Some(first) = call.arguments.first() {
            if let Some(specifier) = string_literal_argument(first) {
                collector.push(specifier, ImportKind::Require, call.span.start as usize);
            }
        }
    } else if let Some((module, export)) = direct_require_member_call(collector, &call.callee) {
        collector.record_explicit_import_call(module, export, call.span.start as usize, "member");
    } else if let Some(scope) = immediately_invoked_function_scope(collector, &call.callee) {
        collector.record_local_invocation(scope, call.span.start as usize);
    } else if let Some(callee) = simple_callee_name(&call.callee) {
        collector.record_binding_aware_invocation(&callee, call.span.start as usize, "direct");
        if collector.should_record_call(&callee) {
            collector.function_calls.push(FunctionCall {
                caller: collector.current_function(),
                static_cwd: static_process_cwd_arg(&callee, &call.arguments),
                callee,
                static_arg: call.arguments.first().and_then(static_path_argument),
            });
        }
    } else {
        let caller = collector.current_function();
        if collector.collect_call_reachability && !collector.suppress_call_reachability {
            collector.call_reachability.push(CallReachabilityFact {
                caller: caller.clone(),
                callee: "dynamic call expression".to_string(),
                binding: CallBinding::Unresolved {
                    display: "dynamic call expression".to_string(),
                },
                line: import_line_at(&collector.line_starts, call.span.start as usize),
                invocation_kind: "unknown",
            });
        }
        if caller.is_none() {
            collector.has_unknown_top_level_call = true;
        }
        collector.unknown_callers.push(caller);
    }
}

/// Constructors are invocation facts just like calls. They deliberately do
/// not go through the legacy `FunctionCall` channel: that channel models
/// import reachability only, while binding-aware consumers need to retain the
/// distinction between `fn()` and `new Fn()`.
fn visit_new_expression_with_imports(collector: &mut ImportCollector, new: &NewExpression<'_>) {
    if let Some(callee) = simple_callee_name(&new.callee) {
        collector.record_binding_aware_invocation(&callee, new.span.start as usize, "construct");
    } else if collector.collect_call_reachability && !collector.suppress_call_reachability {
        collector.call_reachability.push(CallReachabilityFact {
            caller: collector.current_function(),
            callee: "dynamic constructor expression".to_string(),
            binding: CallBinding::Unresolved {
                display: "dynamic constructor expression".to_string(),
            },
            line: import_line_at(&collector.line_starts, new.span.start as usize),
            invocation_kind: "construct",
        });
    }
}
