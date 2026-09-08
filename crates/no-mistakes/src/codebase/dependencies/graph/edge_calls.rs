/// Projects statically resolved lexical calls into the canonical graph.
///
/// This resolves same-file bindings, direct named and namespace imports, and
/// explicit or unambiguous-star re-export chains. Dynamic calls and computed
/// members deliberately remain absent: connecting those by terminal name would
/// make call traversal unsound.
#[derive(Clone)]
struct CallableFileIndex {
    known_scopes: std::collections::HashSet<String>,
    imported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ImportedBinding>,
    exported:
        std::collections::HashMap<String, crate::codebase::dependencies::extract::ExportedBinding>,
    aliases: std::collections::HashMap<(Option<String>, String), String>,
    stars: Vec<String>,
}

impl CallableFileIndex {
    fn from_facts(file: &crate::codebase::ts_source::facts::TsFileFacts) -> Self {
        Self {
            known_scopes: file.callable_scopes.iter().cloned().collect(),
            imported: file
                .imported_bindings
                .iter()
                .filter(|binding| !binding.is_type_only)
                .map(|binding| (binding.local.clone(), binding.clone()))
                .collect(),
            exported: file
                .exported_bindings
                .iter()
                .map(|binding| (binding.exported.clone(), binding.clone()))
                .collect(),
            aliases: file
                .callable_aliases
                .iter()
                .map(|alias| {
                    (
                        (alias.scope.clone(), alias.local.clone()),
                        alias.target.clone(),
                    )
                })
                .collect(),
            stars: file.star_reexport_specifiers.clone(),
        }
    }

    fn resolve_alias(&self, caller: Option<&str>, callee: &str) -> Option<String> {
        if callee.contains('.') {
            return None;
        }
        let mut scope = caller.map(str::to_string);
        let mut visited = std::collections::HashSet::new();
        let mut target = callee.to_string();
        loop {
            let key = (scope.clone(), target.clone());
            let alias = self.aliases.get(&key)?;
            if !visited.insert(key) {
                return None;
            }
            target = alias.clone();
            if self.aliases.contains_key(&(scope.clone(), target.clone())) {
                continue;
            }
            if target.contains('.')
                || self.imported.contains_key(&target)
                || resolve_local_call_scope(scope.as_deref(), &target, &self.known_scopes).is_some()
            {
                return Some(target);
            }
            if let Some(parent) = scope
                .as_deref()
                .and_then(|current| current.rsplit_once('/').map(|(parent, _)| parent))
            {
                scope = Some(parent.to_string());
            } else if scope.is_some() {
                scope = None;
            } else {
                return None;
            }
        }
    }
}

#[derive(Clone, Default)]
struct CallableResolutionIndexes {
    files: dashmap::DashMap<std::path::PathBuf, std::sync::Arc<CallableFileIndex>>,
    exports: dashmap::DashMap<(std::path::PathBuf, String), ExportedCallableResolution>,
}

/// An `export *` branch can prove that a name is exported without proving it
/// is callable. Keep that distinct from an absent name so a callable branch
/// cannot win over a non-callable collision in another barrel.
#[derive(Clone)]
enum ExportedCallableResolution {
    Absent,
    Callable(std::path::PathBuf, String),
    Unknown,
}

impl ExportedCallableResolution {
    fn callable(self) -> Option<(std::path::PathBuf, String)> {
        match self {
            Self::Callable(path, scope) => Some((path, scope)),
            Self::Absent | Self::Unknown => None,
        }
    }
}

impl CallableResolutionIndexes {
    fn file(
        &self,
        facts: &dyn TsFactLookup,
        path: &std::path::Path,
    ) -> Option<std::sync::Arc<CallableFileIndex>> {
        if let Some(index) = self.files.get(path) {
            return Some(index.clone());
        }
        let index = std::sync::Arc::new(CallableFileIndex::from_facts(facts.get_ts_facts(path)?));
        self.files.insert(path.to_path_buf(), index.clone());
        Some(index)
    }
}

fn collect_call_edges_for_core(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
) -> (Vec<Edge>, Vec<ResolvedCallSite>) {
    use rayon::prelude::*;
    let indexes = CallableResolutionIndexes::default();
    edge_inputs
        .graph_files
        .indexable()
        .par_iter()
        .flat_map_iter(|path| {
            let Some(file) = facts.get_ts_facts(path) else {
                return Vec::new();
            };
            let Some(index) = indexes.file(facts, path) else { return Vec::new() };
            file.function_calls
                .iter()
                .map(|call| {
                    let resolved_callee = index
                        .resolve_alias(call.caller.as_deref(), &call.callee)
                        .unwrap_or_else(|| call.callee.clone());
                    let target_identity = call_target_identity(&index, call, &resolved_callee);
                    let target = match target_identity {
                        crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction => {
                            resolve_local_call_scope(call.caller.as_deref(), &resolved_callee, &index.known_scopes)
                                .map(|scope| (path.to_path_buf(), scope.to_string()))
                        }
                        crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport => {
                            resolve_imported_call_scope(edge_inputs, facts, resolver, path, &index, &resolved_callee, &indexes)
                        }
                        crate::codebase::dependencies::extract::CallTargetIdentity::Global
                        | crate::codebase::dependencies::extract::CallTargetIdentity::Unknown => None,
                    };
                    let source = call.caller.as_deref().map_or_else(
                        || NodeId::file_in(&edge_inputs.interner, path),
                        |caller| NodeId::symbol_in(&edge_inputs.interner, path, caller),
                    );
                    let resolved_target = match (target_identity, target) {
                        (crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction, Some((file, scope))) => {
                            ResolvedCallTarget::RepositoryFunction { file, scope }
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport, repository_target) => {
                            if repository_target.is_none()
                                && imported_call_targets_visible_module(
                                    edge_inputs,
                                    resolver,
                                    path,
                                    &index,
                                    &resolved_callee,
                                )
                            {
                                ResolvedCallTarget::Unknown
                            } else {
                                module_export_target(&index, &resolved_callee, repository_target)
                                    .unwrap_or(ResolvedCallTarget::Unknown)
                            }
                        }
                        (crate::codebase::dependencies::extract::CallTargetIdentity::Global, _) => {
                            ResolvedCallTarget::Global { name: call.callee.clone() }
                        }
                        _ => ResolvedCallTarget::Unknown,
                    };
                    let edge = match &resolved_target {
                        ResolvedCallTarget::RepositoryFunction { file, scope }
                        | ResolvedCallTarget::ModuleExport { repository_target: Some((file, scope)), .. } => Some((
                            source,
                            NodeId::symbol_in(&edge_inputs.interner, file, scope),
                            EdgeKind::Call,
                        )),
                        _ => None,
                    };
                    (
                        edge,
                        ResolvedCallSite {
                            file: path.to_path_buf(),
                            caller: call.caller.clone(),
                            source_callee: call.callee.clone(),
                            line: call.line,
                            offset: call.offset,
                            invocation: call.invocation,
                            target: resolved_target,
                        },
                    )
                })
                .collect::<Vec<_>>()
        })
        .fold(|| (Vec::new(), Vec::new()), |mut output, (edge, site)| {
            if let Some(edge) = edge {
                output.0.push(edge);
            }
            output.1.push(site);
            output
        })
        .reduce(|| (Vec::new(), Vec::new()), |mut left, mut right| {
            left.0.append(&mut right.0);
            left.1.append(&mut right.1);
            left
        })
}

/// A non-collapsing, statically resolved call occurrence. Graph edges dedupe
/// adjacency; policy consumers use these records when each source line matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCallSite {
    pub file: std::path::PathBuf,
    pub caller: Option<String>,
    pub line: u32,
    /// Zero-based source byte where the invocation starts. This distinguishes
    /// multiple forbidden calls on the same source line.
    pub offset: u32,
    pub invocation: crate::codebase::dependencies::extract::InvocationKind,
    /// Exact source spelling, kept apart from canonical target selectors.
    pub source_callee: String,
    pub target: ResolvedCallTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedCallTarget {
    Global {
        name: String,
    },
    ModuleExport {
        specifier: String,
        export_path: String,
        repository_target: Option<(std::path::PathBuf, String)>,
    },
    RepositoryFunction {
        file: std::path::PathBuf,
        scope: String,
    },
    Unknown,
}

fn module_export_target(
    file: &CallableFileIndex,
    callee: &str,
    repository_target: Option<(std::path::PathBuf, String)>,
) -> Option<ResolvedCallTarget> {
    let (local, member) = callee
        .split_once('.')
        .map_or((callee, None), |(local, member)| (local, Some(member)));
    let binding = file.imported.get(local)?;
    let export_path = match binding.kind {
        crate::codebase::dependencies::extract::ImportedBindingKind::Namespace => {
            member?.to_string()
        }
        crate::codebase::dependencies::extract::ImportedBindingKind::Named => member
            .map(|member| format!("{}.{member}", binding.imported))
            .unwrap_or_else(|| binding.imported.clone()),
        crate::codebase::dependencies::extract::ImportedBindingKind::Default => member
            .map(|member| format!("default.{member}"))
            .unwrap_or_else(|| "default".to_string()),
    };
    Some(ResolvedCallTarget::ModuleExport {
        specifier: binding.specifier.clone(),
        export_path,
        repository_target,
    })
}

/// A visible repository module with no callable export is a proven-invalid
/// target, not an unresolved external module export. This distinction keeps
/// malformed and ambiguous barrel imports in the unknown-call policy bucket
/// while retaining module/export provenance for dependencies outside the graph.
fn imported_call_targets_visible_module(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    file: &CallableFileIndex,
    callee: &str,
) -> bool {
    let local = callee.split_once('.').map_or(callee, |(local, _)| local);
    file.imported
        .get(local)
        .and_then(|binding| resolver.resolve(&binding.specifier, path))
        .and_then(|target| edge_inputs.graph_files.visible_path(&target))
        .is_some()
}

impl DepGraph {
    pub fn resolved_call_sites(&self) -> &[ResolvedCallSite] {
        &self.resolved_call_sites
    }
}

/// Root selector primitives consumed by call-policy checks. Test catalogs turn
/// into file roots in their prepared owner; this graph layer deliberately does
/// not discover runner configuration a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallRoot {
    File(std::path::PathBuf),
    Module(std::path::PathBuf),
    Function {
        file: std::path::PathBuf,
        symbol: String,
    },
    /// Files selected by the prepared Vitest project catalog for the requested
    /// project set. Catalog parsing remains owned by the request preparation.
    Vitest {
        files: Vec<std::path::PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallTraversal {
    Direct,
    File,
    Transitive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallTrace {
    pub root: NodeId,
    pub target: NodeId,
    pub nodes: Vec<NodeId>,
}

impl DepGraph {
    /// Expand stable file/module/function roots against this prepared graph.
    /// Unknown files or symbols simply produce no roots; configuration
    /// validation belongs to the rule layer that owns diagnostics.
    pub fn expand_call_roots(&self, roots: &[CallRoot]) -> Vec<NodeId> {
        let mut expanded = roots
            .iter()
            .flat_map(|root| match root {
                CallRoot::Module(file) => {
                    let node = NodeId::file(crate::codebase::ts_resolver::normalize_path(file));
                    self.traversal_edges()
                        .forward()
                        .contains_key(&node)
                        .then_some(node)
                        .into_iter()
                        .collect::<Vec<_>>()
                }
                CallRoot::File(file) => self.callable_file_roots(file),
                CallRoot::Function { file, symbol } => {
                    let node = NodeId::symbol(
                        crate::codebase::ts_resolver::normalize_path(file),
                        symbol.as_str(),
                    );
                    self.callable_nodes
                        .contains(&node)
                        .then_some(node)
                        .into_iter()
                        .collect::<Vec<_>>()
                }
                CallRoot::Vitest { files } => files
                    .iter()
                    .flat_map(|file| self.callable_file_roots(file))
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        expanded.sort();
        expanded.dedup();
        expanded
    }

    fn callable_file_roots(&self, file: &std::path::Path) -> Vec<NodeId> {
        let file = crate::codebase::ts_resolver::normalize_path(file);
        let mut nodes = self
            .callable_nodes_by_file
            .get(&file)
            .cloned()
            .unwrap_or_default();
        let module = NodeId::file(&file);
        if self.traversal_edges().forward().contains_key(&module) {
            nodes.push(module);
        }
        nodes.sort();
        nodes.dedup();
        nodes
    }

    /// Follow canonical call edges with deterministic shortest traces. `File`
    /// never crosses a source-file boundary; `Direct` is one call edge; and
    /// `Transitive` follows resolved calls until `max_depth` when supplied.
    pub fn call_traces(
        &self,
        roots: &[NodeId],
        traversal: CallTraversal,
        max_depth: Option<usize>,
    ) -> Vec<CallTrace> {
        let roots = normalize_nodes(roots);
        let edges = self.traversal_edges().forward();
        let mut out = Vec::new();
        for root in roots {
            let root_file = root.as_file().map(std::path::Path::to_path_buf);
            let cap = match traversal {
                CallTraversal::Direct => Some(1),
                _ => max_depth,
            };
            let mut seen = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            seen.insert(root.clone());
            queue.push_back((root.clone(), vec![root.clone()], 0usize));
            while let Some((node, path, depth)) = queue.pop_front() {
                if cap.is_some_and(|limit| depth >= limit) {
                    continue;
                }
                let Some(neighbors) = edges.get(&node) else {
                    continue;
                };
                for (neighbor, kind) in &neighbors.neighbors {
                    if *kind != EdgeKind::Call {
                        continue;
                    }
                    if traversal == CallTraversal::File
                        && neighbor.as_file() != root_file.as_deref()
                    {
                        continue;
                    }
                    if !seen.insert(neighbor.clone()) {
                        continue;
                    }
                    let mut next = path.clone();
                    next.push(neighbor.clone());
                    out.push(CallTrace {
                        root: root.clone(),
                        target: neighbor.clone(),
                        nodes: next.clone(),
                    });
                    queue.push_back((neighbor.clone(), next, depth + 1));
                }
            }
        }
        out.sort_by(|left, right| {
            (&left.root, &left.target, &left.nodes).cmp(&(&right.root, &right.target, &right.nodes))
        });
        out
    }
}

/// Resolves a direct runtime import binding (including one static namespace
/// member) to a callable in a visible local module.
fn resolve_imported_call_scope(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    file: &CallableFileIndex,
    callee: &str,
    indexes: &CallableResolutionIndexes,
) -> Option<(std::path::PathBuf, String)> {
    let (local, requested_export) = match callee.split_once('.') {
        Some((local, member)) if !member.contains('.') => (local, Some(member)),
        Some(_) => return None,
        None => (callee, None),
    };
    let binding = file
        .imported
        .get(local)
        .filter(|binding| match requested_export {
            Some(_) => {
                binding.kind
                    == crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
            None => {
                binding.kind
                    != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
            }
        })?;
    let target_path = resolver.resolve(&binding.specifier, path)?;
    let target_path = edge_inputs.graph_files.visible_path(&target_path)?;
    let export = requested_export.unwrap_or(&binding.imported);
    resolve_exported_callable(
        edge_inputs,
        facts,
        resolver,
        target_path,
        export,
        indexes,
        &mut Vec::new(),
    )
    .callable()
}

/// Follows explicit local exports and named re-exports.  The visited key keeps
/// malformed barrel cycles finite. `export *` resolves only if exactly one
/// canonical target supplies the requested export; ambiguity is unknown.
fn resolve_exported_callable(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: &dyn TsFactLookup,
    resolver: &dyn ImportResolution,
    path: &std::path::Path,
    export: &str,
    indexes: &CallableResolutionIndexes,
    visited: &mut Vec<(std::path::PathBuf, String)>,
) -> ExportedCallableResolution {
    let key = (path.to_path_buf(), export.to_string());
    if let Some(result) = indexes.exports.get(&key) {
        return result.clone();
    }
    if visited.contains(&key) {
        return ExportedCallableResolution::Unknown;
    }
    visited.push(key.clone());
    let Some(file) = indexes.file(facts, path) else {
        indexes
            .exports
            .insert(key, ExportedCallableResolution::Unknown);
        return ExportedCallableResolution::Unknown;
    };
    let result = if let Some(binding) = file.exported.get(export) {
        if let Some(specifier) = &binding.specifier {
            resolver
                .resolve(specifier, path)
                .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path))
                .map(|target_path| {
                    resolve_exported_callable(
                        edge_inputs,
                        facts,
                        resolver,
                        target_path,
                        &binding.local,
                        indexes,
                        visited,
                    )
                })
                .unwrap_or(ExportedCallableResolution::Unknown)
        } else {
            let local = file
                .resolve_alias(None, &binding.local)
                .unwrap_or_else(|| binding.local.clone());
            file.known_scopes
                .contains(&local)
                .then(|| ExportedCallableResolution::Callable(path.to_path_buf(), local.clone()))
                .or_else(|| {
                    let imported = file.imported.get(&local).filter(|imported| {
                        imported.kind != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
                    })?;
                    resolver
                        .resolve(&imported.specifier, path)
                        .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path))
                        .map(|target_path| {
                            resolve_exported_callable(
                                edge_inputs,
                                facts,
                                resolver,
                                target_path,
                                &imported.imported,
                                indexes,
                                visited,
                            )
                        })
                })
                .unwrap_or(ExportedCallableResolution::Unknown)
        }
    } else {
        // A callable's spelling alone is not an export. Keeping this after the
        // explicit binding lookup prevents a private `fn sameName()` in a barrel
        // from satisfying an imported selector. ECMAScript `export *` deliberately
        // excludes `default`; guessing one from a barrel is incorrect.
        if export == "default" {
            ExportedCallableResolution::Absent
        } else {
            let mut candidates = Vec::new();
            let mut has_unknown_candidate = false;
            for specifier in &file.stars {
                // An unresolved or excluded star source can still export this
                // name. It therefore collides with any callable branch just as
                // a visible non-callable source does; treating it as absent
                // would manufacture an unsound call edge.
                let Some(target_path) = resolver.resolve(specifier, path) else {
                    has_unknown_candidate = true;
                    continue;
                };
                let Some(target_path) = edge_inputs.graph_files.visible_path(&target_path) else {
                    has_unknown_candidate = true;
                    continue;
                };
                let mut branch_visited = visited.clone();
                match resolve_exported_callable(
                    edge_inputs,
                    facts,
                    resolver,
                    target_path,
                    export,
                    indexes,
                    &mut branch_visited,
                ) {
                    ExportedCallableResolution::Absent => {}
                    ExportedCallableResolution::Callable(path, scope) => {
                        candidates.push((path, scope))
                    }
                    ExportedCallableResolution::Unknown => has_unknown_candidate = true,
                }
            }
            candidates.sort();
            candidates.dedup();
            if has_unknown_candidate || candidates.len() > 1 {
                ExportedCallableResolution::Unknown
            } else if let Some((path, scope)) = candidates.pop() {
                ExportedCallableResolution::Callable(path, scope)
            } else {
                ExportedCallableResolution::Absent
            }
        }
    };
    indexes
        .exports
        .insert((path.to_path_buf(), export.to_string()), result.clone());
    result
}

fn call_target_identity(
    file: &CallableFileIndex,
    call: &crate::codebase::dependencies::extract::FunctionCall,
    callee: &str,
) -> crate::codebase::dependencies::extract::CallTargetIdentity {
    if callee == call.callee {
        return call.target_identity;
    }
    let binding = callee
        .split_once('.')
        .map_or(callee, |(binding, _)| binding);
    if file.imported.contains_key(binding) {
        crate::codebase::dependencies::extract::CallTargetIdentity::ModuleExport
    } else if resolve_local_call_scope(call.caller.as_deref(), callee, &file.known_scopes).is_some()
    {
        crate::codebase::dependencies::extract::CallTargetIdentity::RepositoryFunction
    } else {
        crate::codebase::dependencies::extract::CallTargetIdentity::Unknown
    }
}

/// Resolves an unqualified name through lexical parent scopes, matching the
/// existing import-reachability scope behavior without another AST pass.
fn resolve_local_call_scope<'a>(
    caller: Option<&str>,
    callee: &str,
    known: &'a std::collections::HashSet<String>,
) -> Option<&'a str> {
    let binding = callee
        .split_once('.')
        .map_or(callee, |(binding, _)| binding);
    if callee.contains('.') && known.contains(binding) {
        return None;
    }
    let mut scope = caller;
    while let Some(current) = scope {
        let candidate = format!("{current}/{callee}");
        if known.contains(candidate.as_str()) {
            return known.get(candidate.as_str()).map(String::as_str);
        }
        let member = callee.replace('.', "/");
        let candidate = format!("{current}/{member}");
        if known.contains(candidate.as_str()) {
            return known.get(candidate.as_str()).map(String::as_str);
        }
        scope = current.rsplit_once('/').map(|(parent, _)| parent);
    }
    known.get(callee).map(String::as_str).or_else(|| {
        known
            .get(callee.replace('.', "/").as_str())
            .map(String::as_str)
    })
}
