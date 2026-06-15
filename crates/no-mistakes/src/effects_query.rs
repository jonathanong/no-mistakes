//! `effects <kind> --entry <file>`: report every transitive call site of a
//! configured set of effect functions/constructors that is reachable from
//! `<entry>` through the import graph.
//!
//! The function/constructor names per `<kind>` come entirely from configuration
//! (`effects.<kind>` in `.no-mistakes.yml`); nothing is hardcoded. Reachability
//! reuses the canonical dependency graph ([`DepGraph::deps_of`]) over runtime
//! import edges, then each reachable file is parsed once to collect matching
//! call sites with line numbers.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use oxc::ast::ast::{CallExpression, Expression, Function, NewExpression, VariableDeclarator};
use oxc_ast_visit::{walk, Visit};
use rayon::prelude::*;
use serde::Serialize;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::{find_tsconfig, load_tsconfig, normalize_path, TsConfig};
use crate::codebase::ts_source::{byte_offset_to_line, relative_slash_path};
use crate::config::v2::load_v2_config;

/// One matched effect call site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EffectCallSite {
    pub file: String,
    pub line: usize,
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
    pub depth: usize,
}

/// The full `effects <kind>` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectsReport {
    pub kind: String,
    pub entry: String,
    pub call_sites: Vec<EffectCallSite>,
    pub by_category: BTreeMap<String, usize>,
}

impl EffectsReport {
    /// Sorted unique matched file paths, for `--format paths`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .call_sites
            .iter()
            .map(|site| site.file.clone())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Edge kinds that represent runtime reachability (code that actually executes
/// when the entry module is imported). Type-only imports are excluded.
fn runtime_edges() -> HashSet<EdgeKind> {
    HashSet::from([EdgeKind::Import, EdgeKind::DynamicImport, EdgeKind::Require])
}

fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    match tsconfig {
        Some(path) => load_tsconfig(path),
        None => match find_tsconfig(root) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

/// Run the `effects <kind>` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    kind: &str,
    entry: &Path,
    categories: &[String],
    depth: Option<usize>,
) -> Result<EffectsReport> {
    let root = normalize_path(root);
    let config = load_v2_config(&root, config_path)?;
    let Some(kind_config) = config.effects.get(kind) else {
        let available: Vec<&str> = config.effects.keys().map(String::as_str).collect();
        bail!(
            "unknown effects kind: {kind} (configured kinds: {})",
            if available.is_empty() {
                "<none>".to_string()
            } else {
                available.join(", ")
            }
        );
    };

    // name -> category label (None for the flat `functions` list).
    let mut names: HashMap<String, Option<String>> = HashMap::new();
    for (category, functions) in &kind_config.categories {
        if !categories.is_empty() && !categories.iter().any(|c| c == category) {
            continue;
        }
        for function in functions {
            names.insert(function.clone(), Some(category.clone()));
        }
    }
    if categories.is_empty() {
        for function in &kind_config.functions {
            names.entry(function.clone()).or_insert(None);
        }
    }
    if names.is_empty() {
        bail!("effects kind `{kind}` has no functions for the requested categories");
    }

    let entry_abs = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        root.join(entry)
    };
    let entry_node = NodeId::File(normalize_path(&entry_abs));

    let tsconfig = resolve_tsconfig(&root, tsconfig)?;
    let graph =
        DepGraph::build_with_plan_and_config(&root, &tsconfig, GraphBuildPlan::all(), config_path)?;
    let allowed = runtime_edges();
    let reachable = graph.deps_of(std::slice::from_ref(&entry_node), depth, Some(&allowed));

    // Map every reachable file (plus the entry itself at depth 0) to its depth.
    let mut file_depths: HashMap<PathBuf, usize> = HashMap::new();
    if let NodeId::File(path) = &entry_node {
        file_depths.insert(path.clone(), 0);
    }
    for entry in &reachable {
        if let NodeId::File(path) = &entry.node {
            file_depths
                .entry(path.clone())
                .and_modify(|existing| *existing = (*existing).min(entry.depth))
                .or_insert(entry.depth);
        }
    }

    let mut call_sites: Vec<EffectCallSite> = file_depths
        .par_iter()
        .flat_map(|(path, depth)| scan_file(&root, path, *depth, &names))
        .collect();
    call_sites.sort();

    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for site in &call_sites {
        let label = site
            .category
            .clone()
            .unwrap_or_else(|| "uncategorized".to_string());
        *by_category.entry(label).or_insert(0) += 1;
    }

    Ok(EffectsReport {
        kind: kind.to_string(),
        entry: relative_slash_path(&root, &entry_abs),
        call_sites,
        by_category,
    })
}

fn scan_file(
    root: &Path,
    path: &Path,
    depth: usize,
    names: &HashMap<String, Option<String>>,
) -> Vec<EffectCallSite> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let rel = relative_slash_path(root, path);
    crate::ast::with_program(path, &source, |program, source| {
        let mut visitor = EffectVisitor {
            source,
            names,
            caller_stack: Vec::new(),
            hits: Vec::new(),
        };
        visitor.visit_program(program);
        visitor
            .hits
            .into_iter()
            .map(|hit| EffectCallSite {
                file: rel.clone(),
                line: hit.line,
                callee: hit.callee,
                category: hit.category,
                caller: hit.caller,
                depth,
            })
            .collect()
    })
    .unwrap_or_default()
}

struct RawHit {
    line: usize,
    callee: String,
    category: Option<String>,
    caller: Option<String>,
}

struct EffectVisitor<'a> {
    source: &'a str,
    names: &'a HashMap<String, Option<String>>,
    caller_stack: Vec<String>,
    hits: Vec<RawHit>,
}

impl EffectVisitor<'_> {
    fn record(&mut self, callee: &Expression<'_>, byte_offset: u32) {
        if let Some((name, category)) = match_callee(callee, self.names) {
            self.hits.push(RawHit {
                line: byte_offset_to_line(self.source, byte_offset as usize) as usize,
                callee: name,
                category,
                caller: self.caller_stack.last().cloned(),
            });
        }
    }
}

impl<'a> Visit<'a> for EffectVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record(&call.callee, call.span.start);
        walk::walk_call_expression(self, call);
    }

    fn visit_new_expression(&mut self, new: &NewExpression<'a>) {
        self.record(&new.callee, new.span.start);
        walk::walk_new_expression(self, new);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let pushed = function
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .inspect(|name| self.caller_stack.push(name.clone()))
            .is_some();
        walk::walk_function(self, function, flags);
        if pushed {
            self.caller_stack.pop();
        }
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        let name = declarator_function_name(declarator);
        if let Some(name) = &name {
            self.caller_stack.push(name.clone());
        }
        walk::walk_variable_declarator(self, declarator);
        if name.is_some() {
            self.caller_stack.pop();
        }
    }
}

/// The binding name of a `const NAME = () => ...` / `const NAME = function...`
/// declarator, used to attribute nested effect calls to a caller.
fn declarator_function_name(declarator: &VariableDeclarator<'_>) -> Option<String> {
    let is_function = matches!(
        declarator.init,
        Some(Expression::ArrowFunctionExpression(_)) | Some(Expression::FunctionExpression(_))
    );
    if !is_function {
        return None;
    }
    match &declarator.id {
        oxc::ast::ast::BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

fn match_callee(
    callee: &Expression<'_>,
    names: &HashMap<String, Option<String>>,
) -> Option<(String, Option<String>)> {
    for candidate in callee_candidates(callee) {
        if let Some(category) = names.get(&candidate) {
            return Some((candidate, category.clone()));
        }
    }
    None
}

fn callee_candidates(expr: &Expression<'_>) -> Vec<String> {
    match expr {
        Expression::Identifier(ident) => vec![ident.name.to_string()],
        Expression::ParenthesizedExpression(parenthesized) => {
            callee_candidates(&parenthesized.expression)
        }
        Expression::StaticMemberExpression(member) => {
            let property = member.property.name.to_string();
            let mut candidates = vec![property.clone()];
            if let Expression::Identifier(object) = &member.object {
                candidates.push(format!("{}.{}", object.name, property));
            }
            candidates
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
