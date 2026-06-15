//! Library entrypoint for the `infra` (Terraform/OpenTofu) command and its N-API
//! parity bindings. Discovers and parses the configured `.tf` files once, then
//! answers the resource-refs / outputs / test-for queries from the fact map.

mod queries;
mod types;

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

use crate::codebase::terraform::{collect_terraform_facts, TerraformFactMap, TfBlockKind};
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::load_v2_config;
use crate::config::v2::schema::TerraformTestConvention;

pub use types::{ModuleOutput, ModuleOutputsResult, OutputConsumer, ResourceRefRow, TestForRow};

/// Parsed, indexed Terraform facts for one repository, plus the inputs needed to
/// answer file-based queries.
pub struct InfraReport {
    root: PathBuf,
    files: Vec<PathBuf>,
    facts: TerraformFactMap,
    test: TerraformTestConvention,
}

/// Discover and parse the configured Terraform files once.
pub fn analyze_project(root: &Path, config_path: Option<&Path>) -> Result<InfraReport> {
    let root = normalize_path(root);
    // Propagate errors so an explicit but missing/invalid `--config` is reported
    // instead of silently producing an empty result.
    let config = load_v2_config(&root, config_path)?;
    let terraform = config.infra.terraform.clone();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = collect_terraform_facts(&root, &files, &terraform);
    Ok(InfraReport {
        root,
        files,
        facts,
        test: terraform.test,
    })
}

impl InfraReport {
    /// Resources/modules/outputs that reference `address` (e.g. `aws_x.y`).
    pub fn resource_refs(&self, address: &str) -> Vec<ResourceRefRow> {
        let mut rows: Vec<ResourceRefRow> = self
            .facts
            .refs_to
            .get(address)
            .into_iter()
            .flatten()
            .map(|reference| ResourceRefRow {
                address: reference.from_addr.clone(),
                file: self.rel(&reference.from_file),
            })
            .collect();
        rows.sort();
        rows.dedup();
        rows
    }

    /// Outputs a module exports plus the root/parent files that consume them.
    pub fn outputs(&self, module_dir: &str) -> ModuleOutputsResult {
        let dir = normalize_path(&self.root.join(module_dir));
        let mut exports: Vec<ModuleOutput> = self
            .facts
            .outputs_by_module
            .get(&dir)
            .into_iter()
            .flatten()
            .map(|name| ModuleOutput {
                name: name.clone(),
                references: self.output_value_refs(&dir, name),
            })
            .collect();
        exports.sort();

        let mut consumers = Vec::new();
        for (to_addr, source_dir) in &self.facts.module_sources {
            if source_dir != &dir {
                continue;
            }
            for reference in self.facts.refs_to.get(to_addr).into_iter().flatten() {
                if let Some(output) = &reference.module_output {
                    consumers.push(OutputConsumer {
                        output: output.clone(),
                        from: reference.from_addr.clone(),
                        file: self.rel(&reference.from_file),
                    });
                }
            }
        }
        consumers.sort();
        consumers.dedup();

        ModuleOutputsResult {
            module: self.rel(&dir),
            exports,
            consumers,
        }
    }

    fn output_value_refs(&self, dir: &Path, name: &str) -> Vec<String> {
        for file in self.facts.files.values() {
            if file.module_dir.as_path() != dir {
                continue;
            }
            for block in &file.blocks {
                if matches!(block.kind, TfBlockKind::Output) && block.name == name {
                    return block.value_refs.clone();
                }
            }
        }
        Vec::new()
    }

    fn rel(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn build_globset(globs: &[String]) -> Option<GlobSet> {
    if globs.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob).ok()?);
    }
    builder.build().ok()
}

#[cfg(test)]
mod tests;
