use super::*;

pub(super) fn forwarded_alias(collector: &ImportCollector, key: &(usize, String)) -> bool {
    // Ordinary reference traversal already forwards this alias with its saved
    // acquisition coordinates. Retaining it again would duplicate root work.
    collector
        .value_binding_indexes
        .get(&key.0)
        .and_then(|names| names.get(&key.1))
        .is_some_and(|indices| {
            indices.iter().any(|index| {
                matches!(
                    &collector.value_flow.bindings[*index].value,
                    ValueExpression::Named { name, .. } if !name.contains('.')
                )
            })
        })
}

/// Project shared call origins through a foreign member provider's path.
pub(super) struct MemberArguments {
    origins: std::sync::Arc<super::super::super::parameter_call_origins::ParameterCallOrigins>,
}

impl MemberArguments {
    pub(super) fn new(
        origins: std::sync::Arc<super::super::super::parameter_call_origins::ParameterCallOrigins>,
    ) -> Self {
        Self { origins }
    }

    pub(super) fn expand(
        &self,
        references: &mut Vec<(nested::ReceiverWrite, Option<CallableId>, (u32, u32))>,
    ) -> Vec<(nested::ReceiverWrite, Option<CallableId>, (u32, u32))> {
        let (mut arguments, mut possible, mut visited) = (Vec::new(), Vec::new(), fx_set());
        let mut pending: Vec<_> = references
            .iter()
            .cloned()
            .map(|(reference, owner, interval)| (reference, owner, interval, Vec::new(), false))
            .collect();
        while let Some(((receiver, reads, _), owner, interval, mut active, uncertain)) =
            pending.pop()
        {
            let values = self.origins.values(&receiver.0);
            let extra = self.origins.possible_values(&receiver.0);
            if (values.is_none() && extra.is_none()) || active.contains(&receiver.0) {
                continue;
            }
            if !visited.insert((
                receiver.clone(),
                reads.clone(),
                owner,
                interval,
                active.clone(),
                uncertain,
            )) {
                continue;
            }
            active.push(receiver.0.clone());
            for (values, added) in [(values, false), (extra, true)] {
                for value in values.into_iter().flatten() {
                    let mut projected = Vec::new();
                    nested::write_receiver_references(value, &receiver.1, &reads, &mut projected);
                    for reference in projected {
                        let item = (reference, owner, interval);
                        let uncertain = uncertain || added;
                        pending.push((item.0.clone(), owner, interval, active.clone(), uncertain));
                        if uncertain {
                            possible.push(item);
                        } else {
                            arguments.push(item);
                        }
                    }
                }
            }
        }
        references.extend(arguments);
        possible
    }
}
