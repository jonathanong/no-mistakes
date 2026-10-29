#[derive(Default)]
struct RootMockBoundaries {
    blocked: FxHashSet<PathBuf>,
    factories: Vec<NodeId>,
}

impl DepGraph {
    fn project_root_mocks(
        &mut self,
        inputs: &GraphEdgeBuildInputs<'_>,
        facts: Option<&dyn TsFactLookup>,
        resolver: &dyn ImportResolution,
    ) {
        let Some(facts) = facts else { return };
        self.root_mock_boundaries = inputs
            .graph_files
            .indexable()
            .par_iter()
            .filter_map(|path| {
                self.project_file_mocks(inputs, facts, resolver, path)
                    .map(|boundaries| (path.clone(), boundaries))
            })
            .collect();
    }

    fn project_file_mocks(
        &self,
        inputs: &GraphEdgeBuildInputs<'_>,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        path: &Path,
    ) -> Option<RootMockBoundaries> {
        let file = facts.get_ts_facts(path)?;
        let module = file.module_bindings.as_ref()?;
        if module.module_mock_calls.is_empty() {
            return None;
        }
        let safe = file.operational_error.is_none()
            && file.parse_error.is_none()
            && module
                .module_mock_calls
                .iter()
                .all(|call| call.specifier.is_some());
        let scopes: FxHashMap<_, _> = file
            .callable_scope_ids
            .iter()
            .map(|(id, scope)| (id.0, (*id, scope.as_str())))
            .collect();
        let arguments: FxHashMap<_, _> = file
            .function_calls
            .iter()
            .filter_map(|call| call.static_arg.as_deref().map(|arg| (call.offset, arg)))
            .collect();
        let bypasses: Vec<_> = self
            .call_sites_in_file(path)
            .iter()
            .filter(|site| root_mock_actual_bypass(site))
            .map(|site| {
                if root_mock_dynamic_bypass(site) {
                    return None;
                }
                arguments
                    .get(&site.offset)
                    .and_then(|specifier| resolver.resolve(specifier, path))
                    .and_then(|target| {
                        inputs
                            .graph_files
                            .visible_path(&target)
                            .map(Path::to_path_buf)
                    })
            })
            .collect();
        let mut candidates: FxHashMap<PathBuf, bool> = fx_map();
        let mut boundaries = RootMockBoundaries::default();
        for call in &module.module_mock_calls {
            let factories: Vec<_> = call
                .factory_span
                .iter()
                .flat_map(|span| {
                    scopes
                        .iter()
                        .filter(move |(start, _)| span.start <= **start && **start < span.end)
                        .map(|(_, (id, scope))| {
                            NodeId::callable_in(&inputs.interner, path, scope, *id)
                        })
                })
                .collect();
            if call.hoisted {
                boundaries.factories.extend(factories.iter().cloned());
            }
            let Some(target) = call
                .specifier
                .as_deref()
                .and_then(|specifier| resolver.resolve(specifier, path))
                .and_then(|target| inputs.graph_files.visible_path(&target))
            else {
                continue;
            };
            let factory_known = call
                .factory_span
                .as_ref()
                .is_none_or(|span| scopes.contains_key(&span.start));
            let full = inputs.plan.imports
                && target != path
                && safe
                && bypasses.iter().all(|bypass| {
                    bypass
                        .as_deref()
                        .is_some_and(|file| !self.import_reaches_mock(file, target))
                })
                && call.full
                && factory_known
                && self.mock_factory_is_full(&factories, target, facts)
                && call.factory_span.as_ref().is_none_or(|span| {
                    self.mock_factory_imports_are_full(
                        path,
                        span,
                        target,
                        module,
                        resolver,
                        inputs.graph_files,
                    )
                });
            *candidates.entry(target.to_path_buf()).or_insert(true) &= full;
        }
        boundaries.blocked.extend(
            candidates
                .into_iter()
                .filter_map(|(target, full)| full.then_some(target)),
        );
        boundaries.factories.sort();
        boundaries.factories.dedup();
        (!boundaries.blocked.is_empty() || !boundaries.factories.is_empty()).then_some(boundaries)
    }
}
