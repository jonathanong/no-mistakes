//! One prepared source hop for a returned first-parameter builder.
use super::Root;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings,
    helper_records,
    syntax::{Head, Syntax},
    HeadProof, Span,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeSet;
mod source;
use source::source_proof;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub span: Span,
    pub specifier: String,
    pub imported: String,
    pub proof: HeadProof,
}

pub(in super::super) fn module_projection(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    module: Option<&TypeScriptModuleFacts>,
    executors: &FxHashSet<Span>,
) -> (BTreeSet<String>, Vec<Pending>) {
    if syntax.returned_params.is_empty() && syntax.returned_param_roots.is_empty() {
        return Default::default();
    }
    let Some(module) = module else {
        return Default::default();
    };
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut locals: FxHashMap<&str, Option<&ModuleBinding>> = FxHashMap::default();
    for binding in module
        .bindings
        .iter()
        .filter(|b| b.runtime && !b.imported && roots.contains(&b.scope_id))
    {
        locals
            .entry(binding.name.as_str())
            .and_modify(|owner| *owner = None)
            .or_insert(Some(binding));
    }
    let mut imports: FxHashMap<usize, Option<(&str, &str)>> = FxHashMap::default();
    for import in module.imports.iter().filter(|item| !item.type_only) {
        for binding in import
            .bindings
            .iter()
            .filter(|item| !item.type_only && item.imported != "*")
        {
            imports
                .entry(binding.binding_id)
                .and_modify(|entry| *entry = None)
                .or_insert(Some((import.specifier.as_str(), binding.imported.as_str())));
        }
    }
    let dynamic = {
        let mut values = syntax.dynamic_refs.clone();
        values.sort_unstable();
        values
    };
    let mut exported = BTreeSet::new();
    // Export aliases share one exact owner proof, including failed censuses.
    let mut source_proofs = FxHashMap::default();
    let mut seen = FxHashSet::default();
    let mut ambiguous = FxHashSet::default();
    for export in module.exports.iter().filter(|item| !item.type_only) {
        if !seen.insert(export.exported.as_str()) {
            exported.remove(export.exported.as_str());
            ambiguous.insert(export.exported.as_str());
        }
        if ambiguous.contains(export.exported.as_str()) || export.specifier.is_some() {
            continue;
        }
        let Some(owner) = locals.get(export.local.as_str()).copied().flatten() else {
            continue;
        };
        let declaration = (owner.span.start, owner.span.end);
        let Some(candidate) = syntax.returned_params.get(&declaration) else {
            continue;
        };
        if *source_proofs
            .entry(owner.id)
            .or_insert_with(|| source_proof(syntax, bindings, owner, candidate, &dynamic))
        {
            exported.insert(export.exported.clone());
        }
    }
    let mut pending = Vec::new();
    for call in syntax
        .calls
        .iter()
        .filter(|call| executors.contains(&call.span))
    {
        let Some(owner) =
            helper_records::candidate(syntax, bindings, call).filter(|owner| owner.imported)
        else {
            continue;
        };
        let Head::FieldReference(reference) = call.head else {
            continue;
        };
        let Some(result) = bindings.owner(reference) else {
            continue;
        };
        let declaration = (result.span.start, result.span.end);
        let Some(root) = syntax.returned_param_roots.get(&declaration) else {
            continue;
        };
        let Some((_, initializer)) = syntax.record_initializers.get(&declaration) else {
            continue;
        };
        if !bindings.declaration_safe(*initializer) || !bindings.declaration_safe(call.span) {
            continue;
        }
        if !caller_root(bindings, owner, root) {
            continue;
        }
        let Some((specifier, imported)) = imports.get(&owner.id).copied().flatten() else {
            continue;
        };
        pending.push(Pending {
            span: call.span,
            specifier: specifier.to_string(),
            imported: imported.to_string(),
            proof: root.proof,
        });
    }
    (exported, pending)
}

fn caller_root(bindings: &Bindings<'_>, callee: &ModuleBinding, root: &Root) -> bool {
    let Some(actual) = bindings.owner(root.callee) else {
        return false;
    };
    let Some(tag) = bindings.owner(root.tag) else {
        return false;
    };
    actual.id == callee.id && tag.imported && bindings.trusted_import(tag.id)
}
