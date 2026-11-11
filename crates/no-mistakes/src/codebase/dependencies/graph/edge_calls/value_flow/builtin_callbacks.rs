use super::*;

impl CallableFileIndex {
    pub(in super::super) fn prepare_builtin_callbacks(&mut self, file: &TsFileFacts) {
        use crate::codebase::dependencies::extract::{ValueCall, ValueExpression};
        self.values.builtin_callbacks.extend(file.function_calls.iter()
            .filter(|call| self.values.for_of_calls.contains_key(&call.offset))
            .cloned());
        for call in &file.value_flow.calls {
            if matches!(&call.callee, ValueExpression::Named { name, scope: None, .. }
                if matches!(name.as_str(), "setTimeout" | "setInterval" | "setImmediate")
                    && !self.imported.contains_key(name)) {
                if let Some(ValueExpression::Named { offset, .. }) = call.arguments.first() {
                    self.values.native_callbacks.insert(*offset);
                }
                if let Some(ValueExpression::Callable(id)) = call.arguments.first() {
                    self.values.native_callbacks.insert(id.0);
                }
            }
        }
        let lines: FxHashMap<_, _> = file
            .function_calls
            .iter()
            .map(|call| (call.offset, call.line))
            .chain(
                file.unknown_calls
                    .iter()
                    .map(|call| (call.offset, call.line)),
            )
            .collect();
        for call in &file.value_flow.calls {
            let Some(positions) = file.builtin_callback_calls
                .get(&(call.offset, call.end)).copied().flatten() else {
                continue;
            };
            for position in positions.iter() {
                if call.spread_at.is_some_and(|spread| *position >= spread) { continue; }
                let Some(argument) = call.arguments.get(*position) else {
                    continue;
                };
                let (name, scope, offset, end, identity) = match argument {
                    ValueExpression::Named {
                        name,
                        scope,
                        offset,
                        ..
                    } => {
                        let identity = if matches!(scope, None | Some(0))
                            && self.imported.contains_key(name)
                        {
                            CallTargetIdentity::ModuleExport
                        } else {
                            CallTargetIdentity::Unknown
                        };
                        (
                            name.clone(),
                            *scope,
                            *offset,
                            *offset + name.len() as u32,
                            identity,
                        )
                    }
                    ValueExpression::Callable(id) => {
                        let Some(name) = self.values.scopes.get(id) else {
                            continue;
                        };
                        (
                            name.clone(),
                            None,
                            id.0,
                            id.0,
                            CallTargetIdentity::RepositoryFunction,
                        )
                    }
                    _ => continue,
                };
                let caller = call
                    .context
                    .and_then(|id| self.values.scopes.get(&id))
                    .cloned();
                self.values
                    .sites
                    .entry(offset)
                    .or_default()
                    .insert(name.clone(), (offset, end));
                self.values.native_callbacks.insert(offset);
                let forwarded = match (call.source_callee.rsplit('.').next(), positions) {
                    (Some("run"), [1]) => Some(2),
                    (Some("exit"), [0]) => Some(1),
                    _ => None,
                };
                if forwarded.is_some() {
                    self.values.forwarded_callbacks.insert(offset);
                }
                self.values.calls.insert(
                    (offset, end),
                    ValueCall {
                        offset,
                        end,
                        context: call.context,
                        source_callee: name.clone(),
                        callee: argument.clone(),
                        arguments: std::sync::Arc::new(
                            forwarded.map_or_else(Vec::new, |start| call.arguments[start..].to_vec()),
                        ),
                        last_argument_literal: false,
                        bound: false,
                        spread_at: forwarded.and_then(|start| call.spread_at.map(|spread| spread - start)),
                        once: false,
                        repeated: call.repeated,
                    },
                );
                if scope.is_none() {
                    self.values.synthetic_callbacks.insert(name.clone());
                }
                self.values.builtin_callbacks.push(FunctionCall {
                    caller: caller.clone(),
                    caller_id: call.context,
                    syntactic_caller: caller,
                    callee: name,
                    line: lines.get(&call.offset).copied().unwrap_or(0),
                    offset,
                    is_callback: true,
                    invocation: InvocationKind::Callback,
                    target_identity: identity,
                    callee_binding_scope: scope,
                    static_arg: None,
                    static_cwd: None,
                });
            }
        }
    }
}
