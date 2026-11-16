use super::*;
use crate::codebase::dependencies::extract::ImportedBindingKind;

pub(super) fn configured(
    opts: &CompiledOptions,
    facts: &CheckFactMap,
    file: &Path,
    flow: &ValueFlowFacts,
    call: &ValueCall,
) -> bool {
    if call.bound || call.spread_at.is_some() {
        return false;
    }
    let [callback] = call.arguments.as_slice() else {
        return false;
    };
    let ValueExpression::Named { name, scope, .. } = callback else {
        return false;
    };
    let scope = scope.unwrap_or(0);
    if flow
        .identifier_writes
        .iter()
        .any(|write| write.scope == scope && write.name == *name)
        || flow.bindings.iter().any(|binding| {
            binding.scope == scope
                && binding.name == *name
                && (!binding.immutable || binding.invalidated_at.is_some())
        })
    {
        return false;
    }
    opts.identifier_validators.iter().any(|validator| {
        validator.argument == 0
            && match &validator.module {
                None => validator.name == *name,
                Some(module) => {
                    scope == 0
                        && !flow
                            .bindings
                            .iter()
                            .any(|binding| binding.scope == scope && binding.name == *name)
                        && facts.get_ts_facts(file).is_some_and(|ts| {
                            ts.imported_bindings.iter().any(|binding| {
                                !binding.is_type_only
                                    && binding.local == *name
                                    && binding.kind == ImportedBindingKind::Named
                                    && binding.imported == validator.name
                                    && binding.specifier == *module
                            })
                        })
                }
            }
    })
}

pub(super) fn literal(flow: &ValueFlowFacts, call: &ValueCall) -> bool {
    if call.bound || call.spread_at.is_some() {
        return false;
    }
    let [ValueExpression::Callable(owner)] = call.arguments.as_slice() else {
        return false;
    };
    flow.catalogs
        .callbacks
        .get(owner)
        .is_some_and(|summary| summary.reads.iter().all(Vec::is_empty))
}
