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
    /// Whether any file declares a namespace; without one, no import can
    /// lead to a namespace member and the lookups are skipped.
    namespaces_in_repo: bool,
    /// Namespace uses found while resolving this file's calls.
    escapes: std::cell::RefCell<FxHashSet<(std::path::PathBuf, String)>>,
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
        let target_identity = match call_target_identity(index, call, &resolved_callee.callee) {
            CallTargetIdentity::ModuleExport
                if self.import_hidden_by_body(call, &resolved_callee.callee) =>
            {
                CallTargetIdentity::Unknown
            }
            identity => identity,
        };
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
        if let Some((target, id)) = self.namespace_member(call, target_identity) {
            return self.finish(call, target, Some(id));
        }
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
            .filter(|target| *target != ResolvedCallTarget::Unknown)
            .or_else(|| {
                (self.namespaces_in_repo && call.invocation == InvocationKind::Construct)
                    .then(|| self.imported_namespace_member(&resolved_callee.callee))
                    .flatten()
            })
            .unwrap_or(ResolvedCallTarget::Unknown),
            (CallTargetIdentity::Global, _) => ResolvedCallTarget::Global {
                name: call.callee.clone(),
            },
            _ => ResolvedCallTarget::Unknown,
        };
        self.finish(call, resolved_target, callable_id)
    }

    /// Calls whose callee the extractor could not name, unless a traversable
    /// call already records the same site.
    fn unknown_sites(&self, file: &TsFileFacts) -> Vec<ResolvedCallSite> {
        let recorded = file
            .function_calls
            .iter()
            .filter(|call| is_traversable_call(self.index, call))
            .map(|call| (call.caller_id, call.offset, call.invocation))
            .collect::<FxHashSet<_>>();
        file.unknown_calls
            .iter()
            .filter(|call| !recorded.contains(&(call.caller_id, call.offset, call.invocation)))
            .map(|call| ResolvedCallSite {
                file: self.path.to_path_buf(),
                caller: call.caller.clone(),
                caller_id: call.caller_id,
                source_callee: "<unknown>".to_string(),
                line: call.line,
                offset: call.offset,
                invocation: call.invocation,
                target: ResolvedCallTarget::Unknown,
            })
            .collect()
    }

    /// The statically named `extends` base of every class in `file`, resolved
    /// like a call target. The base's synthetic construct record is not a
    /// source call, so it becomes an `Extends` edge and a class declaration
    /// instead of a `Call` edge and a call site.
    fn class_bases(&self, file: &TsFileFacts) -> (Vec<Edge>, Vec<ClassDeclaration>) {
        let lines: FxHashMap<_, _> = file.class_declaration_lines.iter().copied().collect();
        let exported: FxHashSet<&str> =
            file.exported_functions.iter().map(String::as_str).collect();
        let ambient: FxHashSet<_> = file.namespaces.unreported_class_ids.iter().collect();
        let members: FxHashMap<_, _> = file
            .namespaces
            .members
            .iter()
            .map(|member| (member.id, member))
            .collect();
        let mut edges = Vec::new();
        let mut classes = Vec::new();
        for (call, (class_id, scope)) in file
            .function_calls
            .iter()
            .filter_map(|call| class_base_owner(call).map(|owner| (call, owner)))
        {
            let (edge, site) = self.resolve(call);
            edges.extend(edge.map(|(class, base, _)| (class, base, EdgeKind::Extends)));
            let member = members.get(&class_id);
            classes.push(ClassDeclaration {
                file: self.path.to_path_buf(),
                scope: scope.to_string(),
                callable_id: class_id,
                line: lines.get(&class_id).copied().unwrap_or(0),
                exported: member.map_or_else(|| exported.contains(scope), |member| member.exported),
                ambient: ambient.contains(&class_id),
                namespace: member
                    .and_then(|member| member.path.rsplit_once('.'))
                    .map(|(namespace, _)| namespace.to_string()),
                namespace_escaped: false,
                global_base: match site.target {
                    ResolvedCallTarget::Global { name } => Some(name),
                    _ => None,
                },
            });
        }
        (edges, classes)
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
