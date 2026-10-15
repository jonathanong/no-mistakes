use super::*;

impl AssignedReceiverOrigins {
    pub(in super::super) fn record(
        &mut self,
        root: usize,
        path: ValueMutationPath,
        value: ValueExpression,
        store: Store<'_>,
    ) {
        let Store {
            owner,
            reads,
            completed,
            complete,
            stable_alias,
            capture,
            root_identity,
            parent_identities,
        } = store;
        let identities = self.identities(root, root_identity, &path, reads, &parent_identities);
        let timeline = self
            .roots
            .entry(root)
            .or_default()
            .entry(root_identity)
            .or_default()
            .entry(path)
            .or_default()
            .entry(owner)
            .or_default()
            .entry(identities)
            .or_default();
        let identity = if complete {
            Some(completed)
        } else {
            timeline
                .events
                .last_key_value()
                .and_then(|(_, previous)| previous.identity)
        };
        let repeated_alias = !complete && stable_alias && timeline.alias_seen(&value);
        timeline.remember_alias(&value, complete, stable_alias);
        let value = if repeated_alias {
            timeline
                .events
                .last_key_value()
                .expect("previous stable RHS alias")
                .1
                .value
                .clone()
        } else if !complete
            && matches!(
                value,
                ValueExpression::Unknown | ValueExpression::Call(_, _)
            )
        {
            timeline
                .events
                .last_key_value()
                .map_or(ValueExpression::Unknown, |(_, previous)| {
                    previous.value.clone()
                })
        } else if complete {
            value
        } else {
            // Unknown overwrites retain prior possibilities without continuing
            // to claim an exact supported replacement.
            timeline
                .events
                .last_key_value()
                .map_or(value.clone(), |(_, previous)| {
                    ValueExpression::Default(
                        std::sync::Arc::new(previous.value.clone()),
                        std::sync::Arc::new(value),
                    )
                })
        };
        timeline.events.insert(
            completed,
            Origin {
                value,
                complete,
                identity,
                capture,
            },
        );
    }
}
