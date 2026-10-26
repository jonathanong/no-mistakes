impl Engine<'_> {
    fn object_values(&mut self, entries: &[Entry], state: State) -> Option<Vec<(State, Value)>> {
        let expressions = entries
            .iter()
            .map(|entry| match entry {
                Entry::Prop(_, value) => Some(value),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        Some(
            self.all(&expressions, state)?
                .into_iter()
                .map(|(state, values)| {
                    let props = entries
                        .iter()
                        .zip(values)
                        .filter_map(|(entry, value)| match entry {
                            Entry::Prop(key, _) => Some((key.clone(), value)),
                            _ => None,
                        })
                        .collect();
                    (state, Value::Object(props))
                })
                .collect(),
        )
    }

    fn array_values(&mut self, items: &[Expr], state: State) -> Option<Vec<(State, Value)>> {
        Some(
            self.all(&items.iter().collect::<Vec<_>>(), state)?
                .into_iter()
                .map(|(state, items)| (state, Value::Array(items)))
                .collect(),
        )
    }
}
