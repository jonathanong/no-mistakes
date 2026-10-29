//! One prepared source hop for a returned first-parameter builder.
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, HeadProof, Span,
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
    for (span, owner, proof) in super::projected(syntax, bindings, module, executors, &roots) {
        let Some((specifier, imported)) = imports.get(&owner).copied().flatten() else {
            continue;
        };
        pending.push(Pending {
            span,
            specifier: specifier.to_string(),
            imported: imported.to_string(),
            proof,
        });
    }
    (exported, pending)
}
