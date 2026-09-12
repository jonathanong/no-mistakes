//! HCL parsing for a single Terraform file via `hcl-rs`.
//!
//! Extracts declared blocks (resource/data/module/output/variable/local). The
//! reference-walking logic lives in [`super::references`]. Parsing is structural
//! only.

use hcl::expr::Expression;
use hcl::structure::{Block, Body, Structure};
use std::path::{Path, PathBuf};

use super::references::{collect_body_refs, push_expr_refs, walk_expr};
use super::{TerraformBlock, TerraformFileFacts, TerraformRef, TfAddr, TfBlockKind};
use crate::codebase::ts_resolver::normalize_path;

/// Parse one `.tf` file into facts. Returns `None` if it cannot be read or parsed.
pub(super) fn parse_tf_file(path: &Path) -> Option<TerraformFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    parse_source(&source, path)
}

/// Parse HCL source already in memory. Returns `None` if it cannot be parsed.
pub(super) fn parse_source(source: &str, path: &Path) -> Option<TerraformFileFacts> {
    let body = hcl::parse(source).ok()?;
    let module_dir = path.parent().map(Path::to_path_buf).unwrap_or_default();

    let mut blocks = Vec::new();
    let mut references = Vec::new();
    for structure in body.iter() {
        if let Structure::Block(block) = structure {
            handle_block(block, path, &module_dir, &mut blocks, &mut references);
        }
    }
    Some(TerraformFileFacts {
        path: path.to_path_buf(),
        module_dir,
        blocks,
        references,
    })
}

fn handle_block(
    block: &Block,
    path: &Path,
    module_dir: &Path,
    blocks: &mut Vec<TerraformBlock>,
    references: &mut Vec<TerraformRef>,
) {
    let identifier = block.identifier.as_str();
    let labels: Vec<&str> = block.labels.iter().map(|label| label.as_str()).collect();
    match identifier {
        "resource" | "data" => {
            let (Some(type_label), Some(name)) = (labels.first(), labels.get(1)) else {
                return;
            };
            let (kind, addr) = if identifier == "data" {
                (TfBlockKind::Data, format!("data.{type_label}.{name}"))
            } else {
                (TfBlockKind::Resource, format!("{type_label}.{name}"))
            };
            blocks.push(TerraformBlock {
                kind,
                addr: addr.clone(),
                name: (*name).to_string(),
                file: path.to_path_buf(),
                module_source_dir: None,
                value_refs: Vec::new(),
            });
            collect_body_refs(&block.body, path, &addr, references, &[]);
        }
        "module" => {
            let Some(name) = labels.first() else { return };
            let addr = format!("module.{name}");
            let module_source_dir = module_source(&block.body, module_dir);
            blocks.push(TerraformBlock {
                kind: TfBlockKind::Module,
                addr: addr.clone(),
                name: (*name).to_string(),
                file: path.to_path_buf(),
                module_source_dir,
                value_refs: Vec::new(),
            });
            collect_body_refs(&block.body, path, &addr, references, &[]);
        }
        "output" => {
            let Some(name) = labels.first() else { return };
            let addr = format!("output.{name}");
            let value_refs = value_refs(&block.body);
            blocks.push(TerraformBlock {
                kind: TfBlockKind::Output,
                addr: addr.clone(),
                name: (*name).to_string(),
                file: path.to_path_buf(),
                module_source_dir: None,
                value_refs,
            });
            collect_body_refs(&block.body, path, &addr, references, &[]);
        }
        "variable" => {
            let Some(name) = labels.first() else { return };
            let addr = format!("var.{name}");
            blocks.push(TerraformBlock {
                kind: TfBlockKind::Variable,
                addr: addr.clone(),
                name: (*name).to_string(),
                file: path.to_path_buf(),
                module_source_dir: None,
                value_refs: Vec::new(),
            });
            collect_body_refs(&block.body, path, &addr, references, &[]);
        }
        "locals" => {
            for structure in block.body.iter() {
                if let Structure::Attribute(attr) = structure {
                    let addr = format!("local.{}", attr.key.as_str());
                    blocks.push(TerraformBlock {
                        kind: TfBlockKind::Local,
                        addr: addr.clone(),
                        name: attr.key.as_str().to_string(),
                        file: path.to_path_buf(),
                        module_source_dir: None,
                        value_refs: Vec::new(),
                    });
                    push_expr_refs(&attr.expr, path, &addr, references, &[]);
                }
            }
        }
        // `terraform`, `provider`, and unknown blocks contribute no declarations.
        _ => {}
    }
}

/// Resolve a `module` block's `source` to a local directory, if it is a relative
/// or absolute filesystem path (registry/remote sources return `None`).
fn module_source(body: &Body, module_dir: &Path) -> Option<PathBuf> {
    for structure in body.iter() {
        if let Structure::Attribute(attr) = structure {
            if attr.key.as_str() == "source" {
                if let Expression::String(source) = &attr.expr {
                    if source.starts_with("./")
                        || source.starts_with("../")
                        || source.starts_with('/')
                    {
                        return Some(normalize_path(&module_dir.join(source)));
                    }
                }
                return None;
            }
        }
    }
    None
}

/// Addresses referenced by an `output` block's `value` attribute.
fn value_refs(body: &Body) -> Vec<TfAddr> {
    for structure in body.iter() {
        if let Structure::Attribute(attr) = structure {
            if attr.key.as_str() == "value" {
                let mut sink = Vec::new();
                walk_expr(&attr.expr, &mut sink, &[]);
                // Keep the module-output suffix so `module.network.zone_id` is not
                // collapsed to a bare `module.network`.
                let mut addrs: Vec<TfAddr> = sink
                    .into_iter()
                    .map(|(addr, output)| match output {
                        Some(output) => format!("{addr}.{output}"),
                        None => addr,
                    })
                    .collect();
                addrs.sort();
                addrs.dedup();
                return addrs;
            }
        }
    }
    Vec::new()
}
