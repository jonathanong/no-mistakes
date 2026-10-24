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
    let selections = profiles
        .iter()
        .map(|profile| {
            let mut selection = Selection::new(profile, module);
            let (typed, unsupported) =
                typed::collect(program, module, profile, &selection.unsupported);
            selection.typed = typed;
            selection.unsupported.extend(unsupported);
            selection
        })
        .collect::<Vec<_>>();
    let mut syntax = Syntax {
        loose_query: profiles.iter().any(|profile| {
            profile.import_specifier.is_empty()
                && profile.executor_names.iter().any(|name| name == "query")
        }),
        ..Default::default()
    };
    for selection in &selections {
        for binding in &module.bindings {
            for reference in &binding.references {
                let at = (reference.span.start, reference.span.end);
                if selection.typed.contains(&binding.id) {
                    syntax.typed_targets.insert(at);
                }
                if selection.selected.contains(&binding.id) || selection.typed.contains(&binding.id)
                {
                    syntax.targets.insert(at);
                }
                if selection.namespaces.contains(&binding.id)
                    || selection.typed.contains(&binding.id)
                {
                    syntax
                        .namespace_targets
                        .entry(at)
                        .or_default()
                        .insert("query".to_string());
                }
            }
        }
    }
    aliases::collect(program, module, &selections, &mut syntax);
    syntax.visit_program(program);
    let mut owners = FxHashMap::default();
    for binding in &module.bindings {
        for usage in &binding.references {
            owners.insert((usage.span.start, usage.span.end), binding);
        }
    }
    let wanted = syntax
        .calls
        .iter()
        .filter_map(|call| match &call.input {
            Input::Reference(at) => owners
                .get(at)
                .map(|binding| (binding.span.start, binding.span.end)),
            _ => None,
        })
        .collect::<FxHashSet<_>>();
    if !wanted.is_empty() {
        Constants {
            wanted: &wanted,
            syntax: &mut syntax,
        }
        .visit_program(program);
    }
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.sql_identifier.syntax_files", 1);
        observer.increment(
            "postgres.sql_identifier.syntax_inputs",
            (syntax.calls.len() + syntax.constants.len()) as u64,
        );
    }
    let mut output = FileFacts::default();
    for (profile, selection) in profiles.iter().zip(selections) {
        if !profile.selects_executors() {
            output.profiles.push((profile.clone(), Vec::new()));
            continue;
        }
        let calls = calls::project(profile, &selection, module, &syntax, &owners, source);
        output.profiles.push((profile.clone(), calls));
    }
    output
}

mod aliases;
mod calls;
