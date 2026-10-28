//! Four local edges may carry one declarative direct-source import obligation.
use super::{
    dependencies::{dependencies, Input},
    imported::Imports,
    Pending, Signature, Span,
};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashMap;

#[derive(Clone)]
pub(super) struct Summary {
    pub signature: Signature,
    pub required: Option<Pending>,
    pub strict: bool,
}
pub(super) fn proven(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    declarations: &FxHashMap<Span, Option<&ModuleBinding>>,
    names: &FxHashMap<usize, Option<(&str, &str)>>,
) -> FxHashMap<(usize, u8), Summary> {
    let calls = syntax
        .effect_calls
        .iter()
        .map(|call| (call.builder, call))
        .collect::<FxHashMap<_, _>>();
    let mut dynamic_refs = syntax.dynamic_refs.clone();
    dynamic_refs.sort_unstable();
    let mut imports = Imports::new(syntax, names);
    let mut candidates = FxHashMap::default();
    for (&declaration, summaries) in &syntax.effect_functions {
        let Some(callable) = declarations.get(&declaration).copied().flatten() else {
            continue;
        };
        for summary in summaries.iter().flatten() {
            let Some(parameter) = declarations.get(&summary.parameter).copied().flatten() else {
                continue;
            };
            let deps = dependencies(
                syntax,
                bindings,
                Input {
                    summary,
                    callable,
                    parameter,
                },
                &dynamic_refs,
                &calls,
                &mut imports,
            );
            candidates
                .entry((callable.id, summary.position))
                .and_modify(|entry| *entry = None)
                .or_insert(deps.map(|deps| (summary.signature, deps)));
        }
    }
    let mut proven = FxHashMap::default();
    // A simultaneous round admits at most one local edge; cycles never seed.
    for _ in 0..=4 {
        let next = candidates
            .iter()
            .filter_map(|(&key, candidate)| {
                if proven.contains_key(&key) {
                    return None;
                }
                let (signature, deps) = candidate.as_ref()?;
                let mut required = deps.imported.clone();
                let mut strict = deps.strict;
                for dependency in &deps.local {
                    let known: &Summary = proven.get(dependency)?;
                    if !known.signature.plain() {
                        return None;
                    }
                    strict &= known.strict;
                    if let Some(imported) = &known.required {
                        if required.is_some() {
                            return None;
                        }
                        required = Some(imported.clone());
                    }
                }
                if required.is_some() && !strict {
                    return None;
                }
                Some((
                    key,
                    Summary {
                        signature: *signature,
                        required,
                        strict,
                    },
                ))
            })
            .collect::<Vec<_>>();
        if next.is_empty() {
            break;
        }
        proven.extend(next);
    }
    proven
}
