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
    let mut markdown = false;
    let mut package = false;
    let mut workflow = false;
    let mut root_cargo_visible = false;
    let mut terraform = false;
    for path in all_files {
        let name = path.file_name().and_then(|name| name.to_str());
        let extension = path.extension().and_then(|extension| extension.to_str());
        if plan.markdown && !markdown {
            markdown = matches!(extension, Some("md" | "mdx"));
        }
        if plan.package && !package {
            package = name == Some("package.json");
        }
        if config_matches_graph && (plan.workflow_topology || plan.ci) && !workflow {
            workflow = path
                .parent()
                .is_some_and(|parent| workflow_dirs.iter().any(|dir| dir == parent))
                && extension.is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
                });
        }
        if config_matches_graph && plan.ci && !root_cargo_visible {
            root_cargo_visible = root_cargo.as_ref().is_some_and(|cargo| path == cargo);
        }
        if config_matches_graph && plan.terraform && !terraform {
            terraform = tf_extensions.as_ref().is_some_and(|extensions| {
                extension.is_some_and(|extension| {
                    !extension.eq_ignore_ascii_case("json")
                        && extensions
                            .iter()
                            .any(|wanted| wanted.eq_ignore_ascii_case(extension))
                })
            });
        }
        if (!plan.markdown || markdown)
            && (!plan.package || package)
            && (!config_matches_graph || !(plan.workflow_topology || plan.ci) || workflow)
            && (!config_matches_graph || !plan.ci || root_cargo_visible)
            && (!config_matches_graph || !plan.terraform || terraform)
        {
            break;
        }
    }
    // These producers can only start at the listed visible files. Keep the
    // full graph universe: absence of edges does not remove traversal nodes.
    plan.markdown &= markdown;
    plan.package &= package;
    if !config_matches_graph {
        // Standalone `check` accepts an in-memory rule config while graph
        // settings are discovered separately from `config_path` or disk.
        return;
    }
    plan.ci &= workflow && root_cargo_visible;
    plan.workflow_topology &= workflow;
    plan.terraform &= terraform;
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
