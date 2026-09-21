impl PreparedScope {
    fn resolve_check_dependencies_report(&self, request: &AnalyzeReportRequest) -> Result<Value> {
        for option in request.options.keys() {
            anyhow::ensure!(
                matches!(
                    option.as_str(),
                    "dependencyReportIds" | "root" | "tsconfig" | "config"
                ),
                "resolveCheckDependencies does not accept option `{option}`"
            );
        }
        let ids = request
            .options
            .get("dependencyReportIds")
            .context("dependencyReportIds is required")?
            .as_array()
            .context("dependencyReportIds must be an array of dependency report IDs")?;
        anyhow::ensure!(
            !ids.is_empty(),
            "dependencyReportIds must contain at least one ID"
        );
        let mut grouped = std::collections::BTreeMap::<DerivedClosureKey, DerivedClosureGroup>::new();
        let mut seen_ids = std::collections::HashSet::new();
        let cwd = std::env::current_dir().context("reading current directory")?;
        for id in ids {
            let id = id
                .as_str()
                .filter(|id| !id.is_empty())
                .context("dependencyReportIds must contain non-empty strings")?;
            anyhow::ensure!(
                seen_ids.insert(id),
                "dependencyReportIds contains duplicate ID `{id}`"
            );
            let dependencies = self
                .options
                .reports
                .iter()
                .filter(|candidate| {
                    candidate.report_type == "dependencies" && candidate.id.as_deref() == Some(id)
                })
                .collect::<Vec<_>>();
            anyhow::ensure!(
                dependencies.len() <= 1,
                "dependency report ID `{id}` is ambiguous"
            );
            let dependency = dependencies.into_iter().next().with_context(|| {
                    format!("dependencyReportIds references no dependencies report with id `{id}` in this analysis scope")
                })?;
            let args = super::traverse_args(dependency, &self.options)?;
            anyhow::ensure!(
                !args.include_symbols
                    && !args.relationships.is_empty()
                    && args.relationships.iter().all(|relationship| matches!(
                        relationship,
                        crate::codebase::dependencies::RelationshipArg::Import
                            | crate::codebase::dependencies::RelationshipArg::ImportStatic
                            | crate::codebase::dependencies::RelationshipArg::ImportDynamic
                            | crate::codebase::dependencies::RelationshipArg::ImportType
                            | crate::codebase::dependencies::RelationshipArg::ImportRequire
                            | crate::codebase::dependencies::RelationshipArg::Workspace
                    )),
                "dependency report `{id}` must use import or workspace relationships without includeSymbols"
            );
            let concrete_args = concrete_file_closure_args(&args);
            match grouped.entry(DerivedClosureKey::from_args(&args, &concrete_args)) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(DerivedClosureGroup { concrete_args });
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(concrete_args);
                }
            }
        }
        let mut files = std::collections::BTreeSet::new();
        for group in grouped.values() {
            let result = crate::codebase::dependencies::collect_and_filter_entries_prepared(
                &group.concrete_args,
                Direction::Deps,
                &cwd,
                &self.traversal,
            )?;
            files.extend(
                crate::codebase::dependencies::explicit_existing_entry_files(
                    &group.concrete_args,
                    self.traversal.root(),
                    &cwd,
                ),
            );
            files.extend(result.file_paths().map(ToOwned::to_owned));
        }
        let session = self.traversal.session_arc();
        let visible_paths = self
            .traversal
            .visible_paths()
            .paths_for(self.traversal.root());
        let visible: crate::fx::PathSet = visible_paths.iter().cloned().collect();
        let source_store = self.traversal.source_store();
        let report = crate::codebase::queries::resolve_check::batch_report_from_prepared_facts(
            self.traversal.root(),
            files,
            self.traversal.prepared_facts(),
            &visible,
            &source_store,
            self.options
                .tsconfig
                .as_deref()
                .map(|_| self.traversal.tsconfig()),
            &session,
        )?;
        Ok(crate::cli::json_value(&report))
    }
}

/// Fields whose values affect a dependency closure before the derived resolve
/// check removes presentation-only output projections. Entry files deliberately
/// do not participate: unioning compatible roots has the same reachable set as
/// traversing each root independently.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct DerivedClosureKey {
    root: Option<std::path::PathBuf>,
    tsconfig: Option<std::path::PathBuf>,
    depth: Option<usize>,
    filters: Vec<String>,
    target_modules: Vec<String>,
    tests: Vec<String>,
    relationships: Vec<String>,
    candidate_include: Vec<String>,
    candidate_exclude: Vec<String>,
    paths_projection: bool,
}

impl DerivedClosureKey {
    fn from_args(
        original: &crate::codebase::dependencies::TraverseArgs,
        concrete: &crate::codebase::dependencies::TraverseArgs,
    ) -> Self {
        Self {
            root: original.root.clone(),
            tsconfig: original.tsconfig.clone(),
            depth: original.depth,
            filters: canonical_strings(&concrete.filters),
            // These output projections are deliberately retained in the key.
            // The derived check expands them to concrete files, but reports
            // with different public closure semantics must not be combined.
            target_modules: canonical_strings(&original.target_modules),
            tests: canonical_strings(&original.tests),
            relationships: canonical_relationships(&original.relationships),
            candidate_include: canonical_strings(&original.candidate_include),
            candidate_exclude: canonical_strings(&original.candidate_exclude),
            paths_projection: original.projection.is_paths(),
        }
    }
}

struct DerivedClosureGroup {
    concrete_args: crate::codebase::dependencies::TraverseArgs,
}

impl DerivedClosureGroup {
    fn extend(&mut self, args: crate::codebase::dependencies::TraverseArgs) {
        self.concrete_args.files.extend(args.files);
        self.concrete_args.file_symbols.extend(args.file_symbols);
        self.concrete_args
            .file_entrypoints_are_structured
            .extend(args.file_entrypoints_are_structured);
    }
}

fn canonical_strings(values: &[String]) -> Vec<String> {
    let mut values = values.to_vec();
    values.sort();
    values.dedup();
    values
}

fn canonical_relationships(
    relationships: &[crate::codebase::dependencies::RelationshipArg],
) -> Vec<String> {
    let mut relationships = relationships
        .iter()
        .map(|relationship| relationship.as_str().to_string())
        .collect::<Vec<_>>();
    relationships.sort();
    relationships.dedup();
    relationships
}

/// Undo output projections that hide reachable source files so derived
/// resolve checks still see the concrete file closure.
fn concrete_file_closure_args(
    args: &crate::codebase::dependencies::TraverseArgs,
) -> crate::codebase::dependencies::TraverseArgs {
    let mut concrete_args = args.clone();
    for filter in &mut concrete_args.filters {
        if let Some(folder) = filter.strip_suffix('/') {
            *filter = format!("{folder}/**");
        }
    }
    concrete_args.target_modules.clear();
    concrete_args
}
