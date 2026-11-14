use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_semantic::Semantic;
mod call;
mod require_chains;
mod result;
mod safety;

pub(super) use result::TempReturn;

#[derive(Clone)]
pub(in crate::codebase::ts_source::module_facts::arguments) enum TempFactoryBinding {
    Named { sync: bool, result: TempReturn },
    Module { sync: bool, promises_safe: bool },
}

#[derive(Default)]
pub(in crate::codebase::ts_source::module_facts::arguments) struct TempFactories {
    pub(super) bindings: FxHashMap<usize, TempFactoryBinding>,
    pub(super) direct_loads: FxHashMap<(u32, u32), bool>,
    pub(super) unsafe_sync: bool,
    pub(super) unsafe_promises: bool,
}

pub(in crate::codebase::ts_source::module_facts::arguments) fn temp_factories(
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
) -> TempFactories {
    let mut bindings = FxHashMap::default();
    let loads: FxHashMap<_, _> = facts
        .loads
        .iter()
        .filter(|load| load.kind == "require" && result::node_fs(&load.specifier))
        .map(|load| (load.span.start, (load.span.end, load.specifier.as_str())))
        .collect();
    if !facts.imports.iter().any(|import| {
        !import.type_only
            && import.bindings.iter().any(|binding| {
                !binding.type_only
                    && (result::factory(&binding.imported, &import.specifier)
                        || binding.kind == "named"
                            && binding.imported == "promises"
                            && matches!(import.specifier.as_str(), "fs" | "node:fs")
                        || result::node_fs(&import.specifier)
                            && matches!(binding.kind.as_str(), "namespace" | "default"))
            })
    }) && loads.is_empty()
    {
        return TempFactories::default();
    }
    let symbols: FxHashMap<_, _> = semantic
        .scoping()
        .symbol_ids()
        .map(|symbol| (symbol.index(), symbol))
        .collect();
    let locals: FxHashMap<_, _> = facts
        .bindings
        .iter()
        .map(|binding| ((binding.scope_id, binding.name.as_str()), binding.id))
        .collect();
    let mut unsafe_sync = false;
    let mut unsafe_promises = false;
    for import in &facts.imports {
        if import.type_only || !result::node_fs(&import.specifier) {
            continue;
        }
        let sync = matches!(import.specifier.as_str(), "fs" | "node:fs");
        for binding in &import.bindings {
            if binding.type_only {
                continue;
            }
            let symbol = symbols.get(&binding.binding_id).copied();
            let promises = binding.kind == "named" && binding.imported == "promises" && sync;
            if let Some(result) = (binding.kind == "named")
                .then_some(binding.imported.as_str())
                .and_then(|imported| result::named(imported, sync))
            {
                if symbol.is_some_and(|symbol| {
                    semantic
                        .scoping()
                        .get_resolved_references(symbol)
                        .all(|reference| !reference.is_write())
                }) {
                    bindings.insert(
                        binding.binding_id,
                        TempFactoryBinding::Named { sync, result },
                    );
                }
            } else if promises || matches!(binding.kind.as_str(), "namespace" | "default") {
                if let Some(symbol) = symbol {
                    let module_sync = sync && !promises;
                    let unsafe_family = if module_sync {
                        &mut unsafe_sync
                    } else {
                        &mut unsafe_promises
                    };
                    safety::record_module(
                        semantic,
                        symbol,
                        binding.binding_id,
                        module_sync,
                        &mut bindings,
                        unsafe_family,
                    );
                }
            }
        }
    }
    for binding in &facts.require_bindings {
        let Some(&(load_end, specifier)) = loads.get(&binding.load_start) else {
            continue;
        };
        let Some(&id) = locals.get(&(binding.scope_id, binding.local.as_str())) else {
            continue;
        };
        let Some(&symbol) = symbols.get(&id) else {
            continue;
        };
        match require_chains::selection(semantic, symbol, binding, load_end, specifier) {
            require_chains::Selection::Named(sync, result)
                if semantic
                    .scoping()
                    .get_resolved_references(symbol)
                    .all(|r| !r.is_write()) =>
            {
                bindings.insert(id, TempFactoryBinding::Named { sync, result });
            }
            require_chains::Selection::Module(sync) => {
                let unsafe_family = if sync {
                    &mut unsafe_sync
                } else {
                    &mut unsafe_promises
                };
                safety::record_module(semantic, symbol, id, sync, &mut bindings, unsafe_family);
            }
            _ => {}
        }
    }
    let tracked_loads: FxHashSet<_> = facts
        .require_bindings
        .iter()
        .map(|binding| binding.load_start)
        .collect();
    let direct = require_chains::direct_writes(semantic, &loads, &tracked_loads);
    unsafe_sync |= direct.sync_unsafe;
    unsafe_promises |= direct.promises_unsafe;
    if unsafe_sync {
        unsafe_promises = true;
    }
    bindings.retain(|_, binding| {
        let sync = match binding {
            TempFactoryBinding::Named { sync, .. } => *sync,
            TempFactoryBinding::Module {
                sync,
                promises_safe,
            } => {
                if *sync {
                    *promises_safe = !unsafe_promises;
                }
                *sync
            }
        };
        if sync {
            !unsafe_sync
        } else {
            !unsafe_promises
        }
    });
    let direct_loads = facts
        .loads
        .iter()
        .filter(|load| load.kind == "require" && result::node_fs(&load.specifier))
        .map(|load| {
            (
                (load.span.start, load.span.end),
                matches!(load.specifier.as_str(), "fs" | "node:fs"),
            )
        })
        .collect();
    TempFactories {
        bindings,
        direct_loads,
        unsafe_sync,
        unsafe_promises,
    }
}
