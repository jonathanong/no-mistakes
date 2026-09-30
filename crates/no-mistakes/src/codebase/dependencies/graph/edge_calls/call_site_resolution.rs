use crate::codebase::dependencies::extract::CallTargetIdentity;

/// Everything needed to resolve the calls of one source file. Ordinary call
/// sites and class `extends` bases share this so both follow identical
/// alias, class, import, and re-export rules.
struct CallSiteResolution<'a, 'b> {
    edge_inputs: &'a GraphEdgeBuildInputs<'b>,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    indexes: &'a CallableResolutionIndexes,
    path: &'a std::path::Path,
    index: &'a CallableFileIndex,
}

impl CallSiteResolution<'_, '_> {
    fn resolve(&self, call: &FunctionCall) -> (Option<Edge>, ResolvedCallSite) {
        let (index, path) = (self.index, self.path);
        let resolved_callee = index
            .resolve_alias(
                call.caller.as_deref(),
                call.callee_binding_scope,
                &call.callee,
                call.offset,
                call.caller_id,
                call.invocation,
            )
            .or_else(|| {
                (call.target_identity == CallTargetIdentity::RepositoryFunction)
                    .then(|| {
                        index.resolve_class_binding(
                            call.callee_binding_scope,
                            &call.callee,
                            call.invocation,
                        )
                    })
                    .flatten()
            })
            .or_else(|| {
                index.resolve_this_member(
                    call.caller.as_deref(),
                    call.caller_id,
                    &call.callee,
                    call.invocation,
                )
            })
            .unwrap_or_else(|| ResolvedLocalCallee {
                callee: call.callee.clone(),
                callable_id: None,
            });
        let callable_id = resolved_callee.callable_id.or_else(|| {
            index.resolve_local_callable_id(call.callee_binding_scope, &resolved_callee.callee)
        });
        let target_identity = call_target_identity(index, call, &resolved_callee.callee);
        let target = match target_identity {
            CallTargetIdentity::RepositoryFunction => resolve_local_call_scope(
                call.caller.as_deref(),
                call.callee_binding_scope,
                &resolved_callee.callee,
                &index.known_scopes,
                &index.class_scopes,
            )
            .map(|scope| (path.to_path_buf(), scope.to_string())),
            CallTargetIdentity::ModuleExport
            | CallTargetIdentity::Global
            | CallTargetIdentity::Unknown => None,
        };
        let source = call.caller.as_deref().map_or_else(
            || NodeId::file_in(&self.edge_inputs.interner, path),
            |caller| {
                call.caller_id.map_or_else(
                    || NodeId::symbol_in(&self.edge_inputs.interner, path, caller),
                    |id| NodeId::callable_in(&self.edge_inputs.interner, path, caller, id),
                )
            },
        );
        let resolved_target = match (target_identity, target) {
            (CallTargetIdentity::RepositoryFunction, Some((file, scope))) => {
                ResolvedCallTarget::RepositoryFunction { file, scope }
            }
            (CallTargetIdentity::ModuleExport, _) => resolve_imported_call_target(
                self.edge_inputs,
                self.facts,
                self.resolver,
                path,
                index,
                &resolved_callee.callee,
                self.indexes,
            )
            .unwrap_or(ResolvedCallTarget::Unknown),
            (CallTargetIdentity::Global, _) => ResolvedCallTarget::Global {
                name: call.callee.clone(),
            },
            _ => ResolvedCallTarget::Unknown,
        };
        let edge = graph_call_target_node(
            &self.edge_inputs.interner,
            self.facts,
            self.indexes,
            &resolved_target,
            callable_id,
        )
        .map(|target| (source, target, EdgeKind::Call));
        (
            edge,
            ResolvedCallSite {
                file: path.to_path_buf(),
                caller: call.caller.clone(),
                caller_id: call.caller_id,
                source_callee: call.callee.clone(),
                line: call.line,
                offset: call.offset,
                invocation: call.invocation,
                target: resolved_target,
            },
        )
    }

    /// The statically named `extends` base of every class in `file`. Its
    /// synthetic construct record is not a source call, so it is projected
    /// here instead of into the call graph.
    fn class_bases(&self, file: &TsFileFacts) -> Vec<ResolvedClassBase> {
        let lines: FxHashMap<_, _> = file.class_declaration_lines.iter().copied().collect();
        let exported: FxHashSet<&str> =
            file.exported_functions.iter().map(String::as_str).collect();
        file.function_calls
            .iter()
            .filter_map(|call| class_base_owner(call).map(|owner| (call, owner)))
            .map(|(call, (class_id, class_scope))| {
                let (_, site) = self.resolve(call);
                ResolvedClassBase {
                    file: self.path.to_path_buf(),
                    exported: exported.contains(class_scope),
                    class_scope: class_scope.to_string(),
                    class_id,
                    line: lines.get(&class_id).copied().unwrap_or(0),
                    source_base: site.source_callee,
                    base: site.target,
                }
            })
            .collect()
    }
}

/// A class's base is recorded as a callback-flavoured construct whose caller
/// is the class itself; nothing else produces that combination. Returns the
/// derived class's identity and scope.
fn class_base_owner(
    call: &FunctionCall,
) -> Option<(crate::codebase::dependencies::extract::CallableId, &str)> {
    if !call.is_callback || call.invocation != InvocationKind::Construct {
        return None;
    }
    Some((call.caller_id?, call.caller.as_deref()?))
}
