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
) -> (Vec<SqlCall>, FxHashSet<Span>) {
    let (mut calls, direct, trusted) = direct_calls(
        profile,
        selection,
        syntax,
        owners,
        source,
        !module.diagnostics.is_empty(),
    );
    append_escapes(module, selection, syntax, &direct, source, &mut calls);
    append_loads(module, profile, source, &mut calls);
    append_runtime(selection, syntax, &direct, source, &mut calls);
    calls.sort_by_key(|call| call.span);
    calls.dedup_by_key(|call| call.span);
    (calls, trusted)
}

fn direct_calls(
    profile: &EmbeddedSqlOptions,
    selection: &Selection,
    syntax: &Syntax,
    owners: &Owners<'_>,
    source: &str,
    gaps: bool,
) -> (Vec<SqlCall>, FxHashSet<Span>, FxHashSet<Span>) {
    let Selection {
        selected,
        namespaces,
        unsupported,
        typed,
        ..
    } = selection;
    let mut calls = Vec::new();
    let mut direct = FxHashSet::default();
    let mut trusted = FxHashSet::default();
    for call in &syntax.calls {
        let Some((root, parts)) = &call.callee else {
            continue;
        };
        let owner = owners.get(root).copied();
        let named = owner.is_some_and(|binding| selected.contains(&binding.id)) && parts.len() == 1;
        let alias = (parts.len() == 1 || (parts.len() == 2 && parts[1] == "query"))
            && syntax
                .aliases
                .get(root)
                .is_some_and(|id| selected.contains(id) || typed.contains(id));
        let namespace = owner.is_some_and(|binding| namespaces.contains(&binding.id))
            && parts.len() == 2
            && parts[1] == "query";
        let typed = owner.is_some_and(|binding| typed.contains(&binding.id))
            && (parts.len() == 1
                || (parts.len() == 2 && parts[1] == "query")
                || input::indirect_executor_call(parts));
        // Broad member selection has no positive receiver ownership proof.
        let loose = profile.import_specifier.is_empty()
            && parts.len() > 1
            && parts.last().is_some_and(|name| name == "query")
            && profile.executor_names.iter().any(|name| name == "query");
        if !(named || alias || namespace || loose || typed) {
            continue;
        }
        direct.insert(*root);
        let uncertain = gaps
            || alias
            || (namespace && !typed)
            || loose
            || unsupported
                .iter()
                .any(|id| owner.is_some_and(|binding| binding.id == *id));
        if !uncertain && !input::indirect_executor_call(parts) {
            trusted.insert(call.span);
        }
        let value = if uncertain {
            Input::Unknown
        } else {
            call.input.clone()
        };
        calls.push(SqlCall {
            span: call.span,
            line: crate::codebase::ts_source::byte_offset_to_line(source, call.span.0 as usize),
            input: normalize_input(value, syntax, owners),
        });
    }
    (calls, direct, trusted)
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
        ..
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
            if (namespaces.contains(&binding.id)
                || (typed.contains(&binding.id) && !unsupported.contains(&binding.id)))
                && syntax
                    .namespace_members
                    .get(&at)
                    .is_some_and(|name| name != "query")
            {
                continue;
            }
            // Type annotations and type queries are erased even when their
            // imported binding also denotes an unsupported runtime executor.
            if direct.contains(&at)
                || !usage.runtime
                || selection.transports.contains(&at)
                || syntax
                    .metadata
                    .belongs_to(&at, direct, &syntax.template_binds)
            {
                continue;
            }
            if syntax.alias_initializers.get(&at) == Some(&binding.id) {
                continue;
            }
            calls.push(SqlCall {
                span: at,
                line: syntax.metadata.line(&at, source),
                input: Input::Unknown,
            });
        }
    }
    for (at, owner) in &syntax.aliases {
        if (selected.contains(owner) || typed.contains(owner))
            && !direct.contains(at)
            && !selection.transports.contains(at)
            && !syntax
                .metadata
                .belongs_to(at, direct, &syntax.template_binds)
        {
            calls.push(SqlCall {
                span: *at,
                line: syntax.metadata.line(at, source),
                input: Input::Unknown,
            });
        }
    }
}

include!("calls/loads.rs");

include!("calls/runtime.rs");

include!("calls/input.rs");
