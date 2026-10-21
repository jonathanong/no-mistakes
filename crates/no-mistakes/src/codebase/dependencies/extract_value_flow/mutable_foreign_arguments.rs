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
        while let Some(((receiver, reads, root_read), owner, interval, mut active, uncertain)) =
            pending.pop()
        {
            let selected = self.origins.values_at(&receiver.0, root_read);
            let uncertain = uncertain || selected.uncertain;
            if (selected.values.is_none() && selected.possible.iter().all(Option::is_none))
                || active.contains(&(receiver.0.clone(), root_read))
            {
                continue;
            }
            if !visited.insert((
                receiver.clone(),
                reads.clone(),
                root_read,
                owner,
                interval,
                active.clone(),
                uncertain,
            )) {
                continue;
            }
            active.push((receiver.0.clone(), root_read));
            for (values, added) in [
                (selected.values, false),
                (selected.possible[0], true),
                (selected.possible[1], true),
            ] {
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
