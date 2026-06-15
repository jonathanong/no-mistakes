use crate::codebase::terraform::{TerraformBlock, TerraformFactMap, TfBlockKind};

fn collect_terraform_edges(
    root: &Path,
    all_files: &[PathBuf],
    config_options: Option<&GraphConfigOptions>,
) -> Vec<Edge> {
    let Some(config_options) = config_options else {
        return Vec::new();
    };
    if config_options.terraform.module_roots.is_empty() {
        return Vec::new();
    }
    let facts = crate::codebase::terraform::collect_terraform_facts(
        root,
        all_files,
        &config_options.terraform,
    );
    if facts.files.is_empty() {
        return Vec::new();
    }

    let mut edges = Vec::new();
    collect_terraform_reference_edges(&facts, &mut edges);
    collect_terraform_module_edges(&facts, &mut edges);
    collect_terraform_output_edges(&facts, &mut edges);
    edges
}

/// Module-local references (`<type>.<name>`, `data.<type>.<name>`, `var.<name>`,
/// `local.<name>`) → declaring files. Module references are handled separately.
///
/// Terraform addresses are unique only within a module, so a reference resolves
/// only to declarations in the referencing file's own module directory.
fn collect_terraform_reference_edges(facts: &TerraformFactMap, edges: &mut Vec<Edge>) {
    for refs in facts.refs_to.values() {
        for reference in refs {
            if reference.module_output.is_some() || !is_module_local_addr(&reference.to_addr) {
                continue;
            }
            let Some(from_module) = module_dir_of(facts, &reference.from_file) else {
                continue;
            };
            let Some(targets) = facts.declarations.get(&reference.to_addr) else {
                continue;
            };
            for target in targets {
                if module_dir_of(facts, target) != Some(from_module) {
                    continue;
                }
                push_terraform_edge(
                    edges,
                    &reference.from_file,
                    target,
                    EdgeKind::TerraformReference,
                );
            }
        }
    }
}

fn module_dir_of<'a>(facts: &'a TerraformFactMap, file: &Path) -> Option<&'a Path> {
    facts.files.get(file).map(|file| file.module_dir.as_path())
}

/// `module` blocks → files in the module's local source directory.
fn collect_terraform_module_edges(facts: &TerraformFactMap, edges: &mut Vec<Edge>) {
    for file in facts.files.values() {
        for block in &file.blocks {
            if !matches!(block.kind, TfBlockKind::Module) {
                continue;
            }
            push_module_source_edges(facts, block, edges);
        }
    }
}

fn push_module_source_edges(facts: &TerraformFactMap, block: &TerraformBlock, edges: &mut Vec<Edge>) {
    let Some(source_dir) = &block.module_source_dir else {
        return;
    };
    let Some(target_files) = facts.files_by_module.get(source_dir) else {
        return;
    };
    for target in target_files {
        push_terraform_edge(edges, &block.file, target, EdgeKind::TerraformModuleRef);
    }
}

/// References to a child module: `module.<name>.<output>` links to the file
/// declaring that output; a bare `module.<name>` (e.g. `depends_on`) links to all
/// of the module's files so whole-module dependencies are not lost.
fn collect_terraform_output_edges(facts: &TerraformFactMap, edges: &mut Vec<Edge>) {
    for refs in facts.refs_to.values() {
        for reference in refs {
            let Some(source_dir) = facts.module_sources.get(&reference.to_addr) else {
                continue;
            };
            let Some(module_files) = facts.files_by_module.get(source_dir) else {
                continue;
            };
            match &reference.module_output {
                Some(output) => {
                    let output_addr = format!("output.{output}");
                    let Some(decls) = facts.declarations.get(&output_addr) else {
                        continue;
                    };
                    for target in decls.iter().filter(|file| module_files.contains(*file)) {
                        push_terraform_edge(
                            edges,
                            &reference.from_file,
                            target,
                            EdgeKind::TerraformOutputRef,
                        );
                    }
                }
                None => {
                    for target in module_files {
                        push_terraform_edge(
                            edges,
                            &reference.from_file,
                            target,
                            EdgeKind::TerraformModuleRef,
                        );
                    }
                }
            }
        }
    }
}

/// Whether `addr` names a module-local declaration (resource, data source, var,
/// or local) — i.e. not a `module.` reference (handled by the module/output
/// collectors) or a bare `output.` address (never referenced directly).
fn is_module_local_addr(addr: &str) -> bool {
    !addr.starts_with("module.") && !addr.starts_with("output.")
}

fn push_terraform_edge(edges: &mut Vec<Edge>, source: &Path, target: &Path, kind: EdgeKind) {
    if source != target {
        edges.push((
            NodeId::File(source.to_path_buf()),
            NodeId::File(target.to_path_buf()),
            kind,
        ));
    }
}
