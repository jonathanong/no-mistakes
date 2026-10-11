use super::{
    identities::{contains_identity_writes, independent_argument, Targets},
    Effects,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind,
};

impl Effects<'_, '_> {
    pub(super) fn project_call(
        &mut self,
        call: &CallExpression<'_>,
        effects: Targets,
        escape: Option<bool>,
    ) -> (Targets, Targets) {
        let Some(identities) = &self.identities else {
            return (effects, self.appended_builders(call));
        };
        let mut projected = Vec::new();
        for (name, identity) in effects {
            if let Some(targets) = identities.target(name, identity) {
                projected.extend(targets);
            } else {
                self.uncertain = true;
            }
        }
        if escape.is_none() {
            let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
                unreachable!("prepared parameter helpers have identifier callees");
            };
            let helper = self
                .visitor
                .functions
                .parameter_builder(name.name.as_str())
                .expect("call_builder_effect selected this prepared helper");
            if !helper.fragments.is_empty() {
                for argument in call.arguments.get(helper.parameter_index).into_iter() {
                    let argument = match argument {
                        Argument::SpreadElement(spread) => &spread.argument,
                        argument => argument
                            .as_expression()
                            .expect("non-spread arguments are expressions"),
                    };
                    let preceding_writes = call.arguments[..helper.parameter_index]
                        .iter()
                        .any(argument_writes);
                    if preceding_writes && !independent_argument(self.visitor, argument) {
                        self.uncertain = true;
                    } else if let Some(targets) = identities.value(self.visitor, argument) {
                        projected.extend(targets);
                    } else {
                        self.uncertain = true;
                    }
                }
            }
        }
        let receiver = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                Some(&member.object)
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") => {
                Some(&member.object)
            }
            _ => None,
        };
        if let Some(receiver) = receiver {
            if let Some(targets) = identities.value(self.visitor, receiver) {
                projected.extend(targets);
            } else {
                self.uncertain = true;
            }
        }
        // The prepared consumer classification excludes executors, fresh
        // factories, append calls and pure summarized passthrough helpers.
        if escape.is_some() && !self.visitor.known_builder_consumer(call) {
            let mut preceding_writes = contains_identity_writes(&call.callee);
            for argument in &call.arguments {
                let expression = match argument {
                    Argument::SpreadElement(spread) => Some(&spread.argument),
                    argument => argument.as_expression(),
                }
                .expect("arguments contain expressions");
                // Each argument captures its value before the next argument
                // runs. Prior writes can invalidate a later binding lookup;
                // later writes must not replace an already captured object.
                if preceding_writes && !independent_argument(self.visitor, expression) {
                    self.uncertain = true;
                }
                let mut pending = vec![expression];
                while let Some(expression) = pending.pop() {
                    // A later property/element can read a binding changed by
                    // an earlier one. Entry-time identities cannot prove it.
                    self.uncertain |= matches!(
                        unwrap_ts_wrappers(expression),
                        Expression::ArrayExpression(_) | Expression::ObjectExpression(_)
                    ) && contains_identity_writes(expression);
                    match unwrap_ts_wrappers(expression) {
                        Expression::ArrayExpression(array) => pending.extend(
                            array.elements.iter().filter_map(|element| match element {
                                ArrayExpressionElement::SpreadElement(spread) => {
                                    Some(&spread.argument)
                                }
                                element => element.as_expression(),
                            }),
                        ),
                        Expression::ObjectExpression(object) => pending.extend(
                            object.properties.iter().map(|property| match property {
                                ObjectPropertyKind::ObjectProperty(property) => &property.value,
                                ObjectPropertyKind::SpreadProperty(spread) => &spread.argument,
                            }),
                        ),
                        _ => {
                            if let Some(targets) = identities.value(self.visitor, expression) {
                                projected.extend(targets);
                            }
                            // Uncertain replacements keep their taint through
                            // conditional, logical and sequence value wrappers.
                            else if identities.tainted(expression) {
                                self.uncertain = true;
                            }
                        }
                    }
                }
                preceding_writes |= contains_identity_writes(expression);
            }
        }
        (projected, Vec::new())
    }
}

fn argument_writes(argument: &Argument<'_>) -> bool {
    match argument {
        Argument::SpreadElement(spread) => contains_identity_writes(&spread.argument),
        argument => contains_identity_writes(argument.as_expression().unwrap()),
    }
}
