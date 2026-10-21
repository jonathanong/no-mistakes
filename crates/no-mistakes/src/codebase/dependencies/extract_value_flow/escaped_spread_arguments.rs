use super::*;

#[derive(Default)]
pub(super) struct Arguments {
    pub(super) position: usize,
    pub(super) open_at: Option<usize>,
    pub(super) extras: FxHashMap<usize, Vec<ValueExpression>>,
    pub(super) open_vals: Vec<ValueExpression>,
    guarded_open_tail: bool,
    pub(super) guarded_extras: FxHashMap<usize, Vec<ValueExpression>>,
    pub(super) guarded_open: Vec<ValueExpression>,
}
impl Arguments {
    pub(super) fn ordinary(&mut self, values: Vec<ValueExpression>, raw_index: usize) {
        if self.open_at.is_some() {
            if self.guarded_open_tail {
                self.guarded_open.extend(values);
            } else {
                self.open_vals.extend(values);
            }
        } else if values.len() != 1 || self.position != raw_index {
            if !values.is_empty() {
                self.extras.entry(self.position).or_default().extend(values);
            }
            self.position += 1;
        } else {
            self.position += 1;
        }
    }
    /// Literal recursion retains the ordinary slot contract. Only named guarded
    /// baselines split into the new possible channel.
    pub(super) fn spread(
        &mut self,
        collector: &ImportCollector,
        expression: &Expression<'_>,
        before: u32,
    ) {
        let expression = crate::codebase::ts_source::unwrap_ts_wrappers(expression);
        if let Expression::ArrayExpression(array) = expression {
            for element in &array.elements {
                match element {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(nested) => {
                        self.spread(collector, &nested.argument, before);
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {
                        if self.open_at.is_none() {
                            self.position += 1;
                        }
                    }
                    other => {
                        let values = other.as_expression().map_or_else(Vec::new, |expression| {
                            super::escaped_call_arguments::argument_values(collector, expression)
                        });
                        if self.open_at.is_some() {
                            if self.guarded_open_tail {
                                self.guarded_open.extend(values);
                            } else {
                                self.open_vals.extend(values);
                            }
                        } else {
                            self.extras.entry(self.position).or_default().extend(values);
                            self.position += 1;
                        }
                    }
                }
            }
            return;
        }
        let value = collector.value_expression(expression);
        match super::escaped_guarded_arrays::entries(collector, &value, before) {
            Some(entries) if self.open_at.is_none() => {
                let mut next = self.position;
                for (slot, value, possible) in entries {
                    let at = self.position + slot;
                    let channel = if possible {
                        &mut self.guarded_extras
                    } else {
                        &mut self.extras
                    };
                    channel.entry(at).or_default().push(value);
                    next = next.max(at + 1);
                }
                self.position = next;
            }
            Some(entries) => {
                // An unknown prefix can shift the guarded spread and its suffix
                // beyond a written parameter; these are possible destinations.
                self.guarded_open_tail |= entries.iter().any(|(_, _, possible)| *possible);
                for (_, value, possible) in entries {
                    if possible || self.guarded_open_tail {
                        self.guarded_open.push(value);
                    } else {
                        self.open_vals.push(value);
                    }
                }
            }
            None => {
                self.open_at.get_or_insert(self.position);
            }
        }
    }
}
