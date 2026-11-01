impl DepGraph {
    fn mock_factory_is_full(
        &self,
        factories: &[NodeId],
        root: &Path,
        target: &Path,
        facts: &dyn TsFactLookup,
    ) -> bool {
        let sources = self.call_source_nodes(factories, CallTraversal::Transitive, None);
        let factory_nodes: FxHashSet<_> = factories.iter().collect();
        let mut by_file: FxHashMap<&Path, FxHashSet<&NodeId>> = fx_map();
        for source in &sources {
            let Some(path) = source.as_file() else {
                return false;
            };
            if path == target
                || (!factory_nodes.contains(source) && self.import_reaches_mock(path, target))
            {
                return false;
            }
            by_file.entry(path).or_default().insert(source);
        }
        by_file.into_iter().all(|(path, owners)| {
            let Some(file) = facts.get_ts_facts(path) else {
                return false;
            };
            if file.parse_error.is_some() || file.operational_error.is_some() {
                return false;
            }
            let Some(module) = &file.module_bindings else {
                return false;
            };
            if path != root
                && owners.iter().any(|owner| !factory_nodes.contains(*owner))
                && (module.imports.iter().any(|import| !import.type_only)
                    || !module.loads.is_empty()
                    || !module.diagnostics.is_empty())
            {
                return false;
            }
            self.call_sites_in_file(path).iter().all(|site| {
                let owner = site.caller.as_ref().map_or_else(
                    || NodeId::file(path),
                    |scope| match site.caller_id {
                        Some(id) => NodeId::callable(path, scope.as_str(), id),
                        None => NodeId::symbol(path, scope.as_str()),
                    },
                );
                if !owners.contains(&owner) {
                    return true;
                }
                framework_mock_fn(site)
                    || !site.has_unknown_target()
                        && !site.targets().any(|candidate| {
                        matches!(
                            candidate.target,
                            KnownCallTargetRef::Global { .. }
                                | KnownCallTargetRef::ModuleExport {
                                    repository_target: None,
                                    ..
                                }
                        )
                    })
                    && !module
                        .module_mock_actual_spans
                        .iter()
                        .any(|span| span.start <= site.offset && site.offset < span.end)
                    && !site
                        .target_nodes()
                        .any(|node| node.as_file() == Some(target))
            })
        })
    }
}

fn root_mock_actual_bypass(site: &ResolvedCallSite) -> bool {
    fn bypass(name: &str) -> bool {
        matches!(
            name.rsplit('.').next(),
            Some("importActual" | "requireActual" | "importOriginal" | "eval" | "Function")
        )
    }
    bypass(&site.source_callee)
        || site.targets().any(|candidate| match candidate.target {
            KnownCallTargetRef::Global { name } => bypass(name),
            KnownCallTargetRef::ModuleExport { export_path, .. } => bypass(export_path),
            _ => false,
        })
}

fn root_mock_dynamic_bypass(site: &ResolvedCallSite) -> bool {
    matches!(
        site.source_callee.rsplit('.').next(),
        Some("eval" | "Function")
    ) || site.targets().any(|candidate| {
        matches!(
            candidate.target,
            KnownCallTargetRef::Global {
                name: "eval" | "Function"
            }
        )
    })
}

fn framework_mock_fn(site: &ResolvedCallSite) -> bool {
    matches!(site.source_callee.as_str(), "vi.fn" | "jest.fn")
        && site.targets().count() > 0
        && site.targets().all(|candidate| match candidate.target {
            KnownCallTargetRef::Global { name } => name == site.source_callee,
            KnownCallTargetRef::ModuleExport { specifier, .. } => {
                matches!(specifier, "vitest" | "@jest/globals")
            }
            _ => false,
        })
}
