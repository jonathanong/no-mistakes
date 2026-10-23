use super::*;

/// Capture the actual post-constructor, post-refinement agreement inputs.
/// Canonical edges cannot reconstruct their physical multiplicity or sources.
pub(super) fn capture<'facts>(
    snapshot: &mut ReplacementProbeSnapshot,
    incoming: &ParameterInputs,
    sites: &[ResolvedCallSite],
    indexes: &CallableResolutionIndexes<'facts>,
    facts: &'facts dyn TsFactLookup,
) {
    let mut physical: FxHashMap<_, FxHashMap<_, Vec<_>>> = fx_map();
    for site in sites {
        physical
            .entry(site.file.as_path())
            .or_default()
            .entry(site.offset)
            .or_default()
            .push(site);
    }
    for ((file, owner), arguments) in incoming {
        for argument in arguments {
            let index = indexes.file(facts, &argument.file);
            let source_call = index
                .as_ref()
                .and_then(|index| {
                    index
                        .values
                        .sites
                        .get(&argument.offset)
                        .and_then(|sites| sites.get(&argument.callee))
                        .and_then(|key| index.values.calls.get(key))
                })
                .map(|call| ReplacementSourceCallProbe {
                    file: argument.file.clone(),
                    offset: call.offset,
                    end: call.end,
                    context: call.context.map(|owner| owner.0),
                    bound: call.bound,
                    argument_count: call.arguments.len(),
                    spread_at: call.spread_at,
                    argument_ordinals: (0..call.arguments.len()).collect(),
                });
            let mut physical: Vec<_> = physical
                .get(argument.file.as_path())
                .and_then(|offsets| offsets.get(&argument.offset))
                .into_iter()
                .flatten()
                .filter(|site| {
                    site.source_callee == argument.callee
                        && (site.invocation == InvocationKind::Callback) == argument.callback
                        && (site.invocation == InvocationKind::Callback
                            && index.as_ref().is_some_and(|index| {
                                index
                                    .values
                                    .synthetic_callbacks
                                    .contains(&site.source_callee)
                            }))
                            == argument.synthetic_callback
                })
                .map(|site| ReplacementPhysicalProbe {
                    caller_id: site.caller_id.map(|owner| owner.0),
                    invocation: format!("{:?}", site.invocation),
                })
                .collect();
            physical.sort();
            snapshot.incoming.push(ReplacementIncomingProbe {
                routed_file: file.clone(),
                routed_owner: owner.0,
                caller_file: argument.file.clone(),
                offset: argument.offset,
                callee: argument.callee.clone(),
                callback: argument.callback,
                synthetic_callback: argument.synthetic_callback,
                physical,
                source_call,
            });
        }
    }
    snapshot.incoming.sort();
}
