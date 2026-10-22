fn prune_default_families(
    plan: &mut GraphBuildPlan,
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    config_matches_graph: bool,
) {
    record_inventory_scan!(root);
    let root_cargo = config_matches_graph.then(|| normalize_path(&root.join("Cargo.toml")));
    let workflow_dirs = if config_matches_graph {
        config
            .ci
            .workflow_dirs
            .iter()
            .map(|directory| normalize_path(&root.join(directory)))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let tf = &config.infra.terraform;
    let tf_extensions =
        (config_matches_graph && !tf.module_roots.is_empty()).then(|| tf.effective_extensions());
    let mut found = VisibleDefaultInputs::default();
    for path in all_files {
        let name = path.file_name().and_then(|name| name.to_str());
        let extension = path.extension().and_then(|extension| extension.to_str());
        if plan.markdown && !found.markdown {
            found.markdown = matches!(extension, Some("md" | "mdx"));
        }
        if plan.package && !found.package {
            found.package = name == Some("package.json");
        }
        if config_matches_graph && (plan.workflow_topology || plan.ci) && !found.workflow {
            found.workflow = path
                .parent()
                .is_some_and(|parent| workflow_dirs.iter().any(|dir| dir == parent))
                && extension.is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
                });
        }
        if config_matches_graph && plan.ci && !found.root_cargo_visible {
            found.root_cargo_visible = root_cargo.as_ref().is_some_and(|cargo| path == cargo);
        }
        if config_matches_graph && plan.terraform && !found.terraform {
            found.terraform = tf_extensions.as_ref().is_some_and(|extensions| {
                extension.is_some_and(|extension| {
                    !extension.eq_ignore_ascii_case("json")
                        && extensions
                            .iter()
                            .any(|wanted| wanted.eq_ignore_ascii_case(extension))
                })
            });
        }
        if found.covers(plan, config_matches_graph) {
            break;
        }
    }
    // These producers can only start at the listed visible files. Keep the
    // full graph universe: absence of edges does not remove traversal nodes.
    plan.markdown &= found.markdown;
    plan.package &= found.package;
    if !config_matches_graph {
        // Standalone `check` accepts an in-memory rule config while graph
        // settings are discovered separately from `config_path` or disk.
        return;
    }
    plan.ci &= found.workflow && found.root_cargo_visible;
    plan.workflow_topology &= found.workflow;
    plan.terraform &= found.terraform;
    // Configured .NET solutions/projects and Swift packages may live outside
    // the visible source inventory, so their declarations decide demand.
    plan.dotnet &= !config.tests.dotnet.solutions.is_empty()
        || config
            .tests
            .dotnet
            .projects
            .values()
            .any(|project| !project.project.trim().is_empty());
    plan.swift &= !config.tests.swift.packages.is_empty();
}

#[derive(Default)]
struct VisibleDefaultInputs {
    markdown: bool,
    package: bool,
    workflow: bool,
    root_cargo_visible: bool,
    terraform: bool,
}

impl VisibleDefaultInputs {
    #[inline]
    fn covers(&self, plan: &GraphBuildPlan, config_matches_graph: bool) -> bool {
        (!plan.markdown || self.markdown)
            && (!plan.package || self.package)
            && (!config_matches_graph || !(plan.workflow_topology || plan.ci) || self.workflow)
            && (!config_matches_graph || !plan.ci || self.root_cargo_visible)
            && (!config_matches_graph || !plan.terraform || self.terraform)
    }
}
