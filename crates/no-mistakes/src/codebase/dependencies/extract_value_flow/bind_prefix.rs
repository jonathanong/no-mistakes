use super::*;

pub(super) struct BoundPrefix {
    pub(super) start: usize,
    pub(super) leading: Vec<ValueExpression>,
    pub(super) uncertain: bool,
}

impl ImportCollector {
    /// Remove the first expanded argument (the bound `thisArg`), not the first
    /// syntactic argument. A spread with unknown width may contain that value.
    pub(super) fn bound_prefix(&self, call: &CallExpression<'_>) -> BoundPrefix {
        for (index, argument) in call.arguments.iter().enumerate() {
            let Argument::SpreadElement(spread) = argument else {
                return BoundPrefix {
                    start: index + 1,
                    leading: Vec::new(),
                    uncertain: false,
                };
            };
            let value = self.value_expression(&spread.argument);
            let Some(values) = super::receiver_escapes::escaped_guarded_arrays::exact_entries(
                self,
                &value,
                call.span.start,
            ) else {
                return BoundPrefix {
                    start: index,
                    leading: Vec::new(),
                    uncertain: true,
                };
            };
            let mut values = values.into_iter();
            if values.next().is_some() {
                return BoundPrefix {
                    start: index + 1,
                    leading: values.collect(),
                    uncertain: false,
                };
            }
        }
        BoundPrefix {
            start: call.arguments.len(),
            leading: Vec::new(),
            uncertain: false,
        }
    }
}
