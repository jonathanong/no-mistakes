//! Terraform/OpenTofu structural fact collection.
//!
//! Mirrors the Swift fact collector: discover the configured `.tf` files once,
//! parse each with `hcl-rs` in parallel, and build deterministic reverse indexes
//! that the `infra` queries and the canonical graph edges consume. Parsing is
//! purely structural (HCL syntax, never evaluated), so `for_each`/`count` and
//! remote/registry modules are not expanded — a documented heuristic limit.

mod parse;
mod references;

use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::config::v2::schema::TerraformConfig;

/// A fully-qualified Terraform address used as a stable key, e.g.
/// `aws_route53_record.foo`, `data.aws_ami.ubuntu`, `module.network`,
/// `output.url`, `var.region`, `local.name`.
pub(crate) type TfAddr = String;

/// The kind of declared Terraform block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TfBlockKind {
    Resource,
    Data,
    Module,
    Output,
    Variable,
    Local,
}

/// One declared block (or `locals` entry).
#[derive(Debug, Clone)]
pub(crate) struct TerraformBlock {
    pub kind: TfBlockKind,
    /// Canonical address (see [`TfAddr`]).
    pub addr: TfAddr,
    /// The final name label (`logs`, `network`, `url`, `region`, `name`).
    pub name: String,
    pub file: PathBuf,
    /// For `module` blocks: the resolved local `source` directory, if local.
    pub module_source_dir: Option<PathBuf>,
    /// For `output` blocks: addresses referenced by the `value` expression.
    pub value_refs: Vec<TfAddr>,
}

/// A reference discovered inside an expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TerraformRef {
    pub from_file: PathBuf,
    /// The enclosing block's address.
    pub from_addr: TfAddr,
    /// The referenced declarable address.
    pub to_addr: TfAddr,
    /// For `module.<name>.<output>` references, the output name; else `None`.
    pub module_output: Option<String>,
}

/// Per-file Terraform facts.
#[derive(Debug, Clone, Default)]
pub(crate) struct TerraformFileFacts {
    pub path: PathBuf,
    /// The directory grouping this file (its containing module directory).
    pub module_dir: PathBuf,
    pub blocks: Vec<TerraformBlock>,
    pub references: Vec<TerraformRef>,
}

/// The collected, indexed Terraform fact map for one invocation.
#[derive(Debug, Clone, Default)]
pub(crate) struct TerraformFactMap {
    /// Per-file facts, keyed by absolute path.
    pub files: BTreeMap<PathBuf, TerraformFileFacts>,
    /// Address → files that declare it.
    pub declarations: HashMap<TfAddr, BTreeSet<PathBuf>>,
    /// Address → references that point at it (reverse index for `resource-refs`).
    pub refs_to: HashMap<TfAddr, Vec<TerraformRef>>,
    /// Module directory → output names declared in that module.
    pub outputs_by_module: BTreeMap<PathBuf, BTreeSet<String>>,
    /// `module.<name>` address → resolved local source directory.
    pub module_sources: BTreeMap<TfAddr, PathBuf>,
    /// Module directory → the files comprising that module.
    pub files_by_module: HashMap<PathBuf, BTreeSet<PathBuf>>,
}

/// Discover, parse, and index the configured Terraform files. Returns an empty
/// map (doing no work) unless `module_roots` is configured.
pub(crate) fn collect_terraform_facts(
    root: &Path,
    all_files: &[PathBuf],
    config: &TerraformConfig,
) -> TerraformFactMap {
    if config.module_roots.is_empty() {
        return TerraformFactMap::default();
    }
    // Normalize so the prefix check matches the normalized discovered paths even
    // when `root` contains `..` segments (as test fixture paths do).
    let module_roots: Vec<PathBuf> = config
        .module_roots
        .iter()
        .map(|dir| {
            crate::codebase::ts_resolver::normalize_path(&root.join(dir.trim_end_matches('/')))
        })
        .collect();
    let extensions = config.effective_extensions();

    let tf_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| has_extension(path, &extensions))
        .filter(|path| module_roots.iter().any(|root| path.starts_with(root)))
        .cloned()
        .collect();
    if tf_files.is_empty() {
        return TerraformFactMap::default();
    }

    let mut file_facts: Vec<TerraformFileFacts> = tf_files
        .par_iter()
        .filter_map(|path| parse::parse_tf_file(path))
        .collect();
    file_facts.sort_by(|a, b| a.path.cmp(&b.path));

    build_fact_map(file_facts)
}

fn has_extension(path: &Path, extensions: &[String]) -> bool {
    // `.tf.json` is intentionally excluded: hcl-rs parses HCL native syntax only.
    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        return false;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| extensions.iter().any(|wanted| wanted == ext))
}

fn build_fact_map(file_facts: Vec<TerraformFileFacts>) -> TerraformFactMap {
    let mut facts = TerraformFactMap::default();
    for file in file_facts {
        facts
            .files_by_module
            .entry(file.module_dir.clone())
            .or_default()
            .insert(file.path.clone());
        for block in &file.blocks {
            facts
                .declarations
                .entry(block.addr.clone())
                .or_default()
                .insert(file.path.clone());
            match block.kind {
                TfBlockKind::Output => {
                    facts
                        .outputs_by_module
                        .entry(file.module_dir.clone())
                        .or_default()
                        .insert(block.name.clone());
                }
                TfBlockKind::Module => {
                    if let Some(source) = &block.module_source_dir {
                        facts
                            .module_sources
                            .insert(block.addr.clone(), source.clone());
                    }
                }
                _ => {}
            }
        }
        for reference in &file.references {
            facts
                .refs_to
                .entry(reference.to_addr.clone())
                .or_default()
                .push(reference.clone());
        }
        facts.files.insert(file.path.clone(), file);
    }
    for refs in facts.refs_to.values_mut() {
        refs.sort();
        refs.dedup();
    }
    facts
}

#[cfg(test)]
mod tests;
