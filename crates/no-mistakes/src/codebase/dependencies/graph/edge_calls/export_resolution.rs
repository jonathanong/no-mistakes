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
    visited: &mut ExportResolutionTrace,
) -> ExportedCallableResolution {
    let key = (path.to_path_buf(), export.to_string());
    let cacheable = visited.is_empty();
    if cacheable {
        if let Some(result) = indexes.exports.get(&key) {
            if let Some(failures) = indexes.export_prerequisites.get(&key) {
                visited.failures.extend(failures.iter().cloned());
            }
            return result.clone();
        }
    }
    if visited.contains(&key) {
        // This branch cannot provide the requested export without leaving the
        // cycle. Treat it as absent so another concrete star branch can still
        // prove a unique callable provider.
        return ExportedCallableResolution::Absent;
    }
    visited.record_source(facts, path, indexes.value_imports);
    visited.push(key.clone());
    let Some(file) = indexes.file(facts, path) else {
        if cacheable {
            visited.cache_failures(indexes, &key);
            indexes
                .exports
                .insert(key, ExportedCallableResolution::Unknown);
        }
        return ExportedCallableResolution::Unknown;
    };
    let result = if let Some((namespace, member)) = export
        .split_once('.')
        .filter(|_| !indexes.value_imports || !file.exported.contains_key(export))
    {
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
                if (indexes.value_imports || indexes.external_namespaces) && binding.local == "*" {
                    visited.record_source(facts, target_path, true);
                    ExportedCallableResolution::Namespace(target_path.to_path_buf())
                } else {
                    resolve_exported_callable(
                        edge_inputs,
                        facts,
                        resolver,
                        target_path,
                        &binding.local,
                        indexes,
                        visited,
                    )
                }
            } else if indexes.external_namespaces && binding.local == "*" && external_module_specifier(specifier) {
                ExportedCallableResolution::ExternalNamespace(specifier.clone())
            } else if external_module_specifier(specifier) {
                ExportedCallableResolution::ExternalModuleExport(
                    specifier.clone(),
                    binding.local.clone(),
                )
            } else {
                ExportedCallableResolution::Unknown
            }
        } else {
            let resolved_alias = file.resolve_alias(
                None,
                Some(0),
                &binding.local,
                u32::MAX,
                None,
                crate::codebase::dependencies::extract::InvocationKind::Call,
            );
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
                    resolve_exported_namespace_import(
                        edge_inputs, facts, resolver, path, (file.as_ref(), &local), indexes, visited,
                    )
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
                .unwrap_or_else(|| {
                    if file.values.factory_binding(0, &binding.local)
                        || indexes.value_imports
                            && file.values.immutable_binding(0, &binding.local)
                    {
                        ExportedCallableResolution::Value(path.to_path_buf(), binding.local.clone())
                    } else {
                        ExportedCallableResolution::Unknown
                    }
                })
        }
    } else {
        resolve_star_export(
            edge_inputs,
            facts,
            resolver,
            (path, export, file.as_ref()),
            indexes,
            visited,
        )
    };
    if cacheable {
        visited.cache_failures(indexes, &key);
        indexes.exports.insert(key, result.clone());
    }
    result
}

include!("export_resolution_population.rs");

include!("export_resolution_namespace.rs");

fn external_module_specifier(specifier: &str) -> bool {
    !specifier.starts_with('.') && !specifier.starts_with('/') && !specifier.starts_with('#')
}

include!("export_prerequisites.rs");
include!("export_resolution_stars.rs");
