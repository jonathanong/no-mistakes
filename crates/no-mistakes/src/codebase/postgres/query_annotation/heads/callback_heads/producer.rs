use super::{uses::Uses, Summary};
use crate::codebase::postgres::query_annotation::heads::syntax::Syntax;
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Declaration, Function, Program, Statement};
use std::collections::BTreeMap;

pub(super) fn exported(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    syntax: &Syntax,
) -> BTreeMap<String, Summary> {
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
    let mut result = BTreeMap::new();
    for statement in &program.body {
        let Statement::ExportDeclaration(export) = statement else {
            continue;
        };
        let Declaration::FunctionDeclaration(function) = &export.declaration else {
            continue;
        };
        let Some(id) = &function.id else {
            continue;
        };
        let Some(exported) = exports.get(id.name.as_str()).copied().flatten() else {
            continue;
        };
        let Some(binding) = module.bindings.iter().find(|binding| {
            binding.runtime
                && !binding.imported
                && roots.contains(&binding.scope_id)
                && binding.span.start == id.span.start
                && binding.span.end == id.span.end
        }) else {
            continue;
        };
        if binding.references.iter().any(|usage| {
            let span = (usage.span.start, usage.span.end);
            syntax.assignments.writes.contains_key(&span)
                || syntax.assignments.unsupported_writes.contains(&span)
        }) {
            continue;
        }
        if let Some(summary) = summarize(function) {
            result.insert(exported.to_string(), summary);
        }
    }
    result
}

fn summarize(function: &Function<'_>) -> Option<Summary> {
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
            Uses::inspect(body, names.clone(), callback)
                .map(|payload| Summary { callback, payload })
        })
        .collect::<Vec<_>>();
    let [summary] = summaries.as_slice() else {
        return None;
    };
    Some(*summary)
}
