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
        let mut files = std::collections::BTreeSet::new();
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
            let result = crate::codebase::dependencies::collect_and_filter_entries_prepared(
                &concrete_args,
                Direction::Deps,
                &cwd,
                &self.traversal,
            )?;
            files.extend(
                crate::codebase::dependencies::explicit_existing_entry_files(
                    &args,
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

#[cfg(test)]
mod concrete_file_closure_args_tests {
    use super::concrete_file_closure_args;
    use crate::codebase::dependencies::TraverseArgs;

    #[test]
    fn concrete_file_closure_args_undoes_folder_and_target_module_projections() {
        let mut args = TraverseArgs::default();
        args.filters = vec!["src/**".to_string(), "lib/".to_string()];
        args.target_modules = vec!["@react/*".to_string()];
        let concrete = concrete_file_closure_args(&args);
        assert_eq!(
            concrete.filters,
            vec!["src/**".to_string(), "lib/**".to_string()]
        );
        assert!(concrete.target_modules.is_empty());
        assert_eq!(args.target_modules, vec!["@react/*".to_string()]);
    }
}
