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
    let cacheable = visited.is_empty();
    if cacheable {
        if let Some(result) = indexes.exports.get(&key) {
            return result.clone();
        }
    }
    if visited.contains(&key) {
        // This branch cannot provide the requested export without leaving the
        // cycle. Treat it as absent so another concrete star branch can still
        // prove a unique callable provider.
        return ExportedCallableResolution::Absent;
    }
    visited.push(key.clone());
    let Some(file) = indexes.file(facts, path) else {
        if cacheable {
            indexes
                .exports
                .insert(key, ExportedCallableResolution::Unknown);
        }
        return ExportedCallableResolution::Unknown;
    };
    let result = if let Some((namespace, member)) = export.split_once('.') {
        resolve_exported_namespace_reexport_member(
            edge_inputs,
            facts,
            resolver,
            path,
            (file.as_ref(), namespace, member),
            indexes,
            visited,
        )
    } else if let Some(binding) = file.exported.get(export) {
        if let Some(specifier) = &binding.specifier {
            let visible_target = resolver
                .resolve(specifier, path)
                .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path));
            if let Some(target_path) = visible_target {
                resolve_exported_callable(
                    edge_inputs,
                    facts,
                    resolver,
                    target_path,
                    &binding.local,
                    indexes,
                    visited,
                )
            } else if external_module_specifier(specifier) {
                ExportedCallableResolution::ExternalModuleExport(
                    specifier.clone(),
                    binding.local.clone(),
                )
            } else {
                ExportedCallableResolution::Unknown
            }
        } else {
            let resolved_alias = file.resolve_alias(None, Some(0), &binding.local, u32::MAX, None);
            let local = resolved_alias
                .as_ref()
                .map(|resolved| resolved.callee.clone())
                .unwrap_or_else(|| binding.local.clone());
            (resolved_alias.is_some() && file.known_scopes.contains(&local)
                || file.exported_scopes.contains(&local))
                .then(|| {
                    exported_local_callable(file.as_ref(), path, local.clone(), resolved_alias.as_ref())
                })
                .or_else(|| {
                    resolve_exported_namespace_member_alias(
                        edge_inputs,
                        facts,
                        resolver,
                        path,
                        (file.as_ref(), &local),
                        indexes,
                        visited,
                    )
                })
                .or_else(|| {
                    let imported = file.imported.get(&local).filter(|imported| {
                        imported.kind != crate::codebase::dependencies::extract::ImportedBindingKind::Namespace
                    })?;
                    let visible_target = resolver
                        .resolve(&imported.specifier, path)
                        .and_then(|target_path| edge_inputs.graph_files.visible_path(&target_path));
                    visible_target.map_or_else(
                        || {
                            external_module_specifier(&imported.specifier).then(|| {
                                ExportedCallableResolution::ExternalModuleExport(
                                    imported.specifier.clone(),
                                    imported.imported.clone(),
                                )
                            })
                        },
                        |target_path| {
                            Some(resolve_exported_callable(
                                edge_inputs,
                                facts,
                                resolver,
                                target_path,
                                &imported.imported,
                                indexes,
                                visited,
                            ))
                        },
                    )
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
                    ExportedCallableResolution::Callable(path, scope, callable_id) => {
                        candidates.push((path, scope, callable_id))
                    }
                    ExportedCallableResolution::ExternalModuleExport(_, _)
                    | ExportedCallableResolution::Unknown => has_unknown_candidate = true,
                }
            }
            candidates.sort();
            candidates.dedup();
            if has_unknown_candidate || candidates.len() > 1 {
                ExportedCallableResolution::Unknown
            } else if let Some((path, scope, callable_id)) = candidates.pop() {
                ExportedCallableResolution::Callable(path, scope, callable_id)
            } else {
                ExportedCallableResolution::Absent
            }
        }
    };
    if cacheable {
        indexes.exports.insert(key, result.clone());
    }
    result
}

include!("export_resolution_population.rs");

include!("export_resolution_namespace.rs");

fn external_module_specifier(specifier: &str) -> bool {
    !specifier.starts_with('.') && !specifier.starts_with('/') && !specifier.starts_with('#')
}
