use super::{bindings::Source, *};

enum Step<'a> {
    Cache(SymbolId),
    Member(&'a str),
    Call(bool),
}

impl<'s, 'a> Collector<'s, 'a> {
    // Every supported expression has one provenance input. An explicit stack
    // follows long alias chains without a depth cutoff or recursive Rust calls.
    pub(super) fn origin(&mut self, mut expression: &'s Expression<'a>) -> Option<Origin> {
        let mut steps = Vec::new();
        let mut origin = loop {
            match expression.get_inner_expression() {
                Expression::Identifier(id) => {
                    let Some(reference) = id.reference_id.get() else {
                        break None;
                    };
                    let reference = self.semantic.scoping().get_reference(reference);
                    if !super::super::modules::loads::has_runtime_binding(reference, self.semantic)
                    {
                        break match id.name.as_str() {
                            "module" => Some(Origin::ModuleObject),
                            "require" => Some(Origin::GlobalRequire),
                            _ => None,
                        };
                    }
                    let Some(symbol) = reference.symbol_id() else {
                        break None;
                    };
                    if let Some(origin) = self.bindings.get(&symbol) {
                        break *origin;
                    }
                    // A pending None stops cycles. Mutation never upgrades this
                    // potential origin to a known call target.
                    self.bindings.insert(symbol, None);
                    steps.push(Step::Cache(symbol));
                    match self.binding(symbol) {
                        Some(Source::Known(origin)) => break Some(origin),
                        Some(Source::Alias(value, member)) => {
                            if member {
                                steps.push(Step::Member("createRequire"));
                            }
                            expression = value;
                        }
                        None => break None,
                    }
                }
                Expression::CallExpression(call) => {
                    let module = super::module_argument(call);
                    steps.push(Step::Call(module));
                    expression = &call.callee;
                }
                Expression::ChainExpression(chain) => match &chain.expression {
                    ChainElement::CallExpression(call) => {
                        steps.push(Step::Call(super::module_argument(call)));
                        expression = &call.callee;
                    }
                    ChainElement::StaticMemberExpression(member) => {
                        steps.push(Step::Member(member.property.name.as_str()));
                        expression = &member.object;
                    }
                    ChainElement::ComputedMemberExpression(member) => {
                        let Some(property) = super::computed_name(member) else {
                            break None;
                        };
                        steps.push(Step::Member(property));
                        expression = &member.object;
                    }
                    ChainElement::TSNonNullExpression(inner) => {
                        expression = &inner.expression;
                    }
                    ChainElement::PrivateFieldExpression(_) => break None,
                },
                expression_member => {
                    let Some((object, property)) = member(expression_member) else {
                        break None;
                    };
                    steps.push(Step::Member(property));
                    expression = object;
                }
            }
        };
        for step in steps.into_iter().rev() {
            match step {
                Step::Cache(symbol) => {
                    self.bindings.insert(symbol, origin);
                }
                Step::Member(property) => {
                    origin = match (origin, property) {
                        (Some(Origin::ModuleObject), "require") => Some(Origin::Loader),
                        (Some(Origin::ModuleNamespace), "createRequire") => Some(Origin::Factory),
                        _ => None,
                    }
                }
                Step::Call(module) => {
                    origin = match origin {
                        Some(Origin::Factory) => Some(Origin::Loader),
                        Some(Origin::GlobalRequire | Origin::Loader) if module => {
                            Some(Origin::ModuleNamespace)
                        }
                        _ => None,
                    }
                }
            }
        }
        origin
    }
}
