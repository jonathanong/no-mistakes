use super::{
    candidates::{self, EffectId, Requirement},
    Span, Summary,
};
use crate::codebase::postgres::query_annotation::heads::{bindings::Bindings, syntax::Syntax};
use crate::codebase::ts_source::module_facts::ModuleBinding;
use crate::fx::FxHashMap;

pub(super) fn proven(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    declarations: &FxHashMap<Span, Option<&ModuleBinding>>,
    imports: &FxHashMap<usize, (&str, &str)>,
) -> FxHashMap<EffectId, Summary> {
    let candidates = candidates::collect(syntax, bindings, declarations, imports);
    let mut proven: FxHashMap<EffectId, Summary> = FxHashMap::default();
    for _ in 0..=4 {
        let next = candidates
            .iter()
            .filter_map(|(&key, candidate)| {
                if proven.contains_key(&key) {
                    return None;
                }
                let (signature, deps) = candidate.as_ref()?;
                let required = deps.iter().try_fold(Vec::new(), |mut required, dep| {
                    match dep {
                        Requirement::Import(import) => required.push(import.clone()),
                        Requirement::Local(id, shape) => {
                            let known = proven.get(id).filter(|known| {
                                known.signature.plain() && known.signature.accepts(*shape)
                            })?;
                            required.extend(known.required.iter().cloned());
                        }
                    }
                    Some(required)
                })?;
                Some((
                    key,
                    Summary {
                        signature: *signature,
                        required,
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
