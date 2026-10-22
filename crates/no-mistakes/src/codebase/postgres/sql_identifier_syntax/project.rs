use super::*;

pub(crate) fn collect(
    program: &Program<'_>,
    ts: &TsFileFacts,
    source: &str,
    profiles: &[EmbeddedSqlOptions],
) -> FileFacts {
    let Some(module) = ts.module_bindings.as_deref() else {
        return FileFacts::default();
    };
    let mut syntax = Syntax::default();
    syntax.visit_program(program);
    let mut owners = FxHashMap::default();
    for binding in &module.bindings {
        for usage in &binding.references {
            owners.insert((usage.span.start, usage.span.end), binding);
        }
    }
    let gaps = !module.diagnostics.is_empty();
    let mut output = FileFacts::default();
    for profile in profiles {
        if !profile.selects_executors() {
            output.profiles.push((profile.clone(), Vec::new()));
            continue;
        }
        let mut selected = FxHashMap::default();
        let mut namespaces = FxHashSet::default();
        let mut unsupported = FxHashSet::default();
        for import in &module.imports {
            let exact =
                profile.import_specifier.is_empty() || import.specifier == profile.import_specifier;
            let scoped = exact
                || import
                    .specifier
                    .starts_with(&format!("{}/", profile.import_specifier));
            if !scoped {
                continue;
            }
            for binding in &import.bindings {
                if !profile.import_specifier.is_empty()
                    && binding.kind == "namespace"
                    && !binding.type_only
                {
                    namespaces.insert(binding.binding_id);
                }
                if exact
                    && !binding.type_only
                    && binding.kind == "named"
                    && profile.executor_names.contains(&binding.imported)
                {
                    selected.insert(binding.binding_id, false);
                }
                if !profile.import_specifier.is_empty()
                    && ["withTransaction", "withTransactionOptions"]
                        .contains(&binding.imported.as_str())
                {
                    unsupported.insert(binding.binding_id);
                }
                if profile.executor_factory_names.contains(&binding.imported)
                    || profile.executor_type_names.contains(&binding.imported)
                {
                    unsupported.insert(binding.binding_id);
                }
            }
        }
        let mut calls = Vec::new();
        let mut direct = FxHashSet::default();
        for call in &syntax.calls {
            let Some((root, parts)) = &call.callee else {
                continue;
            };
            let owner = owners.get(root).copied();
            let named =
                owner.is_some_and(|binding| selected.contains_key(&binding.id)) && parts.len() == 1;
            let namespace = owner.is_some_and(|binding| namespaces.contains(&binding.id))
                && parts.len() == 2
                && profile.executor_names.contains(&parts[1]);
            // With no module restriction, explicitly configured member names are a
            // supported selection mode, but ownership is not a positive low proof.
            let loose = profile.import_specifier.is_empty()
                && parts.len() > 1
                && parts
                    .last()
                    .is_some_and(|name| profile.executor_names.contains(name));
            if !(named || namespace || loose) {
                continue;
            }
            direct.insert(*root);
            let mut value = if gaps
                || loose
                || unsupported
                    .iter()
                    .any(|id| owner.is_some_and(|binding| binding.id == *id))
            {
                Input::Unknown
            } else {
                call.input.clone()
            };
            if let Input::Reference(reference) = value {
                value = owners
                    .get(&reference)
                    .filter(|binding| !binding.imported && binding.runtime)
                    .and_then(|binding| {
                        syntax
                            .constants
                            .get(&(binding.span.start, binding.span.end))
                    })
                    .cloned()
                    .unwrap_or(Input::Unknown);
            }
            if let Input::Sql { holes, .. } = &mut value {
                for (_, hole) in holes {
                    if let Some(key) = hole {
                        let reference = syntax.env_references[key];
                        if owners.contains_key(&reference) {
                            *hole = None;
                        } else {
                            *key = key.rsplit_once('@').unwrap().0.to_string();
                        }
                    }
                }
            }
            calls.push(SqlCall {
                span: call.span,
                line: crate::codebase::ts_source::byte_offset_to_line(source, call.span.0 as usize),
                input: value,
            });
        }
        // Reporting the imported reference catches aliases and factories without
        // interpreting their result or silently losing their downstream sinks.
        for binding in &module.bindings {
            if !(selected.contains_key(&binding.id)
                || unsupported.contains(&binding.id)
                || namespaces.contains(&binding.id))
            {
                continue;
            }
            for usage in &binding.references {
                let at = (usage.span.start, usage.span.end);
                if namespaces.contains(&binding.id)
                    && syntax
                        .namespace_members
                        .get(&at)
                        .is_some_and(|name| !profile.executor_names.contains(name))
                {
                    continue;
                }
                if direct.contains(&at) || (!usage.runtime && !unsupported.contains(&binding.id)) {
                    continue;
                }
                calls.push(SqlCall {
                    span: at,
                    line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
                    input: Input::Unknown,
                });
            }
        }
        if profile.selects_executors() {
            for load in &module.loads {
                if (profile.import_specifier.is_empty()
                    || load.specifier == profile.import_specifier)
                    && matches!(load.kind.as_str(), "require" | "dynamicImport")
                {
                    let at = (load.span.start, load.span.end);
                    calls.push(SqlCall {
                        span: at,
                        line: crate::codebase::ts_source::byte_offset_to_line(
                            source,
                            at.0 as usize,
                        ),
                        input: Input::Unknown,
                    });
                }
            }
        }
        calls.sort_by_key(|call| call.span);
        calls.dedup_by_key(|call| call.span);
        output.profiles.push((profile.clone(), calls));
    }
    output
}
