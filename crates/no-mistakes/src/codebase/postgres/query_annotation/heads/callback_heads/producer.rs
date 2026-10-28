use super::{uses::Uses, Summary};
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, Span,
};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Declaration, Function, Program, Statement};
use std::collections::BTreeMap;

pub(super) fn summaries(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    syntax: &Syntax,
    bindings: &Bindings<'_>,
) -> (BTreeMap<String, Summary>, FxHashMap<Span, Summary>) {
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut exports: FxHashMap<&str, Option<&str>> = FxHashMap::default();
    for export in module
        .exports
        .iter()
        .filter(|export| !export.type_only && export.specifier.is_none())
    {
        exports
            .entry(export.local.as_str())
            .and_modify(|value| *value = None)
            .or_insert(Some(export.exported.as_str()));
    }
    let mut owners = FxHashMap::default();
    for binding in module
        .bindings
        .iter()
        .filter(|binding| binding.runtime && !binding.imported && roots.contains(&binding.scope_id))
    {
        owners
            .entry((binding.span.start, binding.span.end))
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding));
    }
    let trusted_tags = syntax
        .append_tags
        .values()
        .filter(|&&tag| {
            bindings
                .owner(tag)
                .is_some_and(|owner| owner.imported && bindings.trusted_import(owner.id))
        })
        .copied()
        .collect::<FxHashSet<_>>();
    let mut exported = BTreeMap::new();
    let mut local = FxHashMap::default();
    for statement in &program.body {
        let function = match statement {
            Statement::FunctionDeclaration(function) => function,
            Statement::ExportDeclaration(export) => {
                let Declaration::FunctionDeclaration(function) = &export.declaration else {
                    continue;
                };
                function
            }
            _ => continue,
        };
        let Some(id) = &function.id else {
            continue;
        };
        let declaration = (id.span.start, id.span.end);
        let Some(binding) = owners.get(&declaration).copied().flatten() else {
            continue;
        };
        if !bindings.declaration_safe(declaration)
            || binding.references.iter().any(|usage| {
                let span = (usage.span.start, usage.span.end);
                !bindings.declaration_safe(span)
                    || syntax.assignments.writes.contains_key(&span)
                    || syntax.assignments.unsupported_writes.contains(&span)
            })
        {
            continue;
        }
        let Some(summary) = summarize(function, &trusted_tags) else {
            continue;
        };
        local.insert(declaration, summary);
        if let Some(name) = exports.get(id.name.as_str()).copied().flatten() {
            exported.insert(name.to_string(), summary);
        }
    }
    (exported, local)
}

fn summarize(function: &Function<'_>, trusted_tags: &FxHashSet<Span>) -> Option<Summary> {
    if function.generator || function.params.rest.is_some() {
        return None;
    }
    let mut names = Vec::new();
    for param in &function.params.items {
        if param.initializer.is_some() || !param.decorators.is_empty() {
            return None;
        }
        let BindingPattern::BindingIdentifier(id) = &param.pattern else {
            return None;
        };
        if names.contains(&id.name.as_str()) {
            return None;
        }
        names.push(id.name.as_str());
    }
    if names.is_empty() || names.len() > 8 {
        return None;
    }
    let body = function.body.as_ref()?;
    let summaries = (0..names.len())
        .filter_map(|callback| {
            Uses::inspect(body, names.clone(), callback, trusted_tags)
                .map(|payload| Summary { callback, payload })
        })
        .collect::<Vec<_>>();
    let [summary] = summaries.as_slice() else {
        return None;
    };
    Some(*summary)
}
