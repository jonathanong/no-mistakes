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
    ) {
        let mut arguments = Vec::new();
        let mut visited = fx_set();
        let mut pending: Vec<_> = references
            .iter()
            .cloned()
            .map(|(reference, owner, interval)| (reference, owner, interval, Vec::new()))
            .collect();
        while let Some(((receiver, reads, _), owner, interval, mut active)) = pending.pop() {
            let Some(values) = self.origins.values(&receiver.0) else {
                continue;
            };
            // A recursive wrapper can append a member on every pass. Its first
            // occurrence remains a possible receiver; revisit no parameter in
            // the same forwarding chain.
            if active.contains(&receiver.0) {
                continue;
            }
            if !visited.insert((
                receiver.clone(),
                reads.clone(),
                owner,
                interval,
                active.clone(),
            )) {
                continue;
            }
            active.push(receiver.0.clone());
            for value in values {
                let mut projected = Vec::new();
                nested::write_receiver_references(value, &receiver.1, &reads, &mut projected);
                for reference in projected {
                    let item = (reference, owner, interval);
                    pending.push((item.0.clone(), owner, interval, active.clone()));
                    arguments.push(item);
                }
            }
        }
        references.extend(arguments);
    }
}
