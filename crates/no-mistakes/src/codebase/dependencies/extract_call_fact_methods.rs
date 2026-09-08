impl ImportCollector {
    fn record_local_invocation(&mut self, scope: String, byte_offset: usize) {
        if !self.collect_call_reachability || self.suppress_call_reachability {
            return;
        }
        self.call_reachability.push(CallReachabilityFact {
            caller: self.current_function(),
            callee: scope.clone(),
            binding: CallBinding::Local { scope },
            line: import_line_at(&self.line_starts, byte_offset),
            invocation_kind: "direct",
        });
    }

    fn record_explicit_import_call(
        &mut self,
        module: String,
        export: String,
        byte_offset: usize,
        invocation_kind: &'static str,
    ) {
        if !self.collect_call_reachability || self.suppress_call_reachability {
            return;
        }
        self.call_reachability.push(CallReachabilityFact {
            caller: self.current_function(),
            callee: export.clone(),
            binding: CallBinding::Import { module, export },
            line: import_line_at(&self.line_starts, byte_offset),
            invocation_kind,
        });
    }

    fn record_binding_aware_invocation(
        &mut self,
        callee: &str,
        byte_offset: usize,
        invocation_kind: &'static str,
    ) {
        if !self.collect_call_reachability || self.suppress_call_reachability {
            return;
        }
        let (binding, member) = callee
            .split_once('.')
            .map_or((callee, None), |(binding, member)| (binding, Some(member)));
        let identity = if let Some((module, export)) = self.visible_call_binding_target(binding) {
            let export = match (export.as_str(), member) {
                ("*", Some(member)) => member.to_string(),
                (_, Some(member)) => format!("{export}.{member}"),
                _ => export,
            };
            CallBinding::Import { module, export }
        } else {
            match self.visible_local_function_scope(callee) {
                Some(Some(scope)) => CallBinding::Local { scope },
                Some(None) => CallBinding::Shadowed {
                    name: callee.to_string(),
                },
                None => CallBinding::Global {
                    name: callee.to_string(),
                },
            }
        };
        self.call_reachability.push(CallReachabilityFact {
            caller: self.current_function(),
            callee: callee.to_string(),
            binding: identity,
            line: import_line_at(&self.line_starts, byte_offset),
            invocation_kind: if invocation_kind == "construct" {
                "construct"
            } else if member.is_some() {
                "member"
            } else {
                invocation_kind
            },
        });
    }

    fn visible_call_binding_target(&self, binding: &str) -> Option<(String, String)> {
        for index in (0..self.local_stack.len()).rev() {
            let locals = &self.local_stack[index];
            let aliases = &self.call_alias_stack[index];
            if let Some(target) = aliases.get(binding) {
                return Some(target.clone());
            }
            if locals.contains(binding)
                || self
                    .call_predeclared_stack
                    .get(index)
                    .is_some_and(|predeclared| predeclared.contains(binding))
            {
                return None;
            }
        }
        self.call_import_bindings.get(binding).cloned()
    }
}

fn immediately_invoked_function_scope(
    collector: &ImportCollector,
    callee: &Expression<'_>,
) -> Option<String> {
    let name = match callee {
        Expression::ArrowFunctionExpression(_) => {
            format!("<anonymous:{}>", collector.anonymous_scope_count + 1)
        }
        Expression::FunctionExpression(function) => function_name(function)
            .unwrap_or_else(|| format!("<anonymous:{}>", collector.anonymous_scope_count + 1)),
        Expression::ParenthesizedExpression(parenthesized) => {
            return immediately_invoked_function_scope(collector, &parenthesized.expression);
        }
        _ => return None,
    };
    Some(
        collector
            .function_stack
            .last()
            .map(|parent| format!("{parent}/{name}"))
            .unwrap_or(name),
    )
}

fn direct_require_member_call(
    collector: &ImportCollector,
    callee: &Expression<'_>,
) -> Option<(String, String)> {
    let Expression::StaticMemberExpression(member) = callee else {
        return None;
    };
    let Expression::CallExpression(require_call) = &member.object else {
        return None;
    };
    let Expression::Identifier(require) = &require_call.callee else {
        return None;
    };
    if !collector.is_builtin_require_binding(require.name.as_str())
        && !collector.is_require_factory(require.name.as_str())
    {
        return None;
    }
    let module = require_call
        .arguments
        .first()
        .and_then(string_literal_argument)?;
    Some((module.to_string(), member.property.name.to_string()))
}
