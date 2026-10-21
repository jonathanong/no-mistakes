use super::*;

impl ImportCollector {
    /// Preserve baseline argument positions and separately retain guarded
    /// alternatives, including possible shifts of arguments after a spread.
    pub(super) fn note_possible_arguments(&mut self, call: &CallExpression<'_>) {
        if !self.receiver_escapes.guarded_arrays {
            return;
        }
        let (mut position, mut open) = (0, None);
        let mut extras: FxHashMap<usize, Vec<ValueExpression>> = fx_map();
        let mut shifted = Vec::new();
        for argument in &call.arguments {
            match argument {
                Argument::SpreadElement(spread) => {
                    let value = self.value_expression(&spread.argument);
                    if let Some((additional, minimum)) =
                        super::escaped_array_bindings::additional_entries(
                            self,
                            &value,
                            call.span.start,
                        )
                    {
                        for (slot, value) in additional {
                            extras.entry(position + slot).or_default().push(value);
                        }
                        open =
                            Some(open.map_or(position + minimum, |at: usize| {
                                at.min(position + minimum)
                            }));
                    }
                    if let Some(entries) =
                        super::escaped_array_entries::array_entries(self, &value, call.span.start)
                    {
                        if open.is_some() {
                            shifted.extend(entries.iter().map(|(_, value)| value.clone()));
                        }
                        position += entries.iter().map(|(slot, _)| slot + 1).max().unwrap_or(0);
                    }
                }
                other => {
                    if open.is_some() {
                        if let Some(expression) = other.as_expression() {
                            shifted.extend(super::escaped_call_arguments::argument_values(
                                self, expression,
                            ));
                        }
                    }
                    position += 1;
                }
            }
        }
        let key = (call.span.start, call.span.end);
        if !extras.is_empty() {
            self.receiver_escapes.possible_escaped.insert(key, extras);
        }
        if let Some(start) = open.filter(|_| !shifted.is_empty()) {
            self.receiver_escapes
                .possible_open
                .insert(key, (start, shifted));
        }
    }
}
