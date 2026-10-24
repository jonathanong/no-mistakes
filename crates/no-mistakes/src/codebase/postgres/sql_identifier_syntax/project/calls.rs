use super::*;
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};

type Owners<'a> = FxHashMap<Span, &'a ModuleBinding>;

pub(super) fn project(
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    module: &TypeScriptModuleFacts,
    syntax: &Syntax,
    owners: &Owners<'_>,
    source: &str,
) -> Vec<SqlCall> {
    let (mut calls, direct) = direct_calls(
        profile,
        selection,
        syntax,
        owners,
        source,
        !module.diagnostics.is_empty(),
    );
    append_escapes(module, selection, syntax, &direct, source, &mut calls);
    append_loads(module, profile, source, &mut calls);
    calls.sort_by_key(|call| call.span);
    calls.dedup_by_key(|call| call.span);
    calls
}

fn direct_calls(
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    syntax: &Syntax,
    owners: &Owners<'_>,
    source: &str,
    gaps: bool,
) -> (Vec<SqlCall>, FxHashSet<Span>) {
    let Selection {
        selected,
        namespaces,
        unsupported,
        typed,
    } = selection;
    let mut calls = Vec::new();
    let mut direct = FxHashSet::default();
    for call in &syntax.calls {
        let Some((root, parts)) = &call.callee else {
            continue;
        };
        let owner = owners.get(root).copied();
        let named = owner.is_some_and(|binding| selected.contains(&binding.id)) && parts.len() == 1;
        let alias = parts.len() == 1
            && syntax
                .aliases
                .get(root)
                .is_some_and(|id| selected.contains(id));
        let namespace = owner.is_some_and(|binding| namespaces.contains(&binding.id))
            && parts.len() == 2
            && parts[1] == "query";
        let typed = owner.is_some_and(|binding| typed.contains(&binding.id))
            && (parts.len() == 1 || (parts.len() == 2 && parts[1] == "query"));
        // Broad member selection has no positive receiver ownership proof.
        let loose = profile.import_specifier.is_empty()
            && parts.len() > 1
            && parts.last().is_some_and(|name| name == "query")
            && profile.executor_names.iter().any(|name| name == "query");
        if !(named || alias || namespace || loose || typed) {
            continue;
        }
        direct.insert(*root);
        let mut value = if gaps
            || alias
            || (namespace && !typed)
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
    (calls, direct)
}

fn append_escapes(
    module: &TypeScriptModuleFacts,
    selection: &Selection,
    syntax: &Syntax,
    direct: &FxHashSet<Span>,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    let Selection {
        selected,
        namespaces,
        unsupported,
        typed,
    } = selection;
    // Reporting the imported reference catches aliases and factories without
    // interpreting their result or silently losing their downstream sinks.
    for binding in &module.bindings {
        if !(selected.contains(&binding.id)
            || unsupported.contains(&binding.id)
            || namespaces.contains(&binding.id)
            || typed.contains(&binding.id))
        {
            continue;
        }
        for usage in &binding.references {
            let at = (usage.span.start, usage.span.end);
            if namespaces.contains(&binding.id)
                && syntax
                    .namespace_members
                    .get(&at)
                    .is_some_and(|name| name != "query")
            {
                continue;
            }
            // Type annotations and type queries are erased even when their
            // imported binding also denotes an unsupported runtime executor.
            if direct.contains(&at) || !usage.runtime {
                continue;
            }
            if syntax.alias_initializers.get(&at) == Some(&binding.id) {
                continue;
            }
            calls.push(SqlCall {
                span: at,
                line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
                input: Input::Unknown,
            });
        }
    }
    for (at, owner) in &syntax.aliases {
        if selected.contains(owner) && !direct.contains(at) {
            calls.push(SqlCall {
                span: *at,
                line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
                input: Input::Unknown,
            });
        }
    }
}

fn append_loads(
    module: &TypeScriptModuleFacts,
    profile: &EmbeddedSqlOptions,
    source: &str,
    calls: &mut Vec<SqlCall>,
) {
    if profile.selects_executors() {
        for load in &module.loads {
            if !profile.import_specifier.is_empty()
                && crate::codebase::postgres::embedded::scoped_bindings::from_configured_module(
                    &load.specifier,
                    &profile.import_specifier,
                )
                && matches!(load.kind.as_str(), "require" | "dynamicImport")
            {
                let at = (load.span.start, load.span.end);
                calls.push(SqlCall {
                    span: at,
                    line: crate::codebase::ts_source::byte_offset_to_line(source, at.0 as usize),
                    input: Input::Unknown,
                });
            }
        }
    }
}
