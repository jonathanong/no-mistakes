use oxc_ast::ast::{
    Argument, ArrayExpressionElement, Expression, IdentifierReference, ObjectPropertyKind,
};

impl super::super::super::ImportCollector {
    pub(in super::super::super) fn mark_closed_registry_allowance(
        &mut self,
        expr: &Expression<'_>,
    ) {
        if !self.collect_value_flow {
            return;
        }
        match expr {
            Expression::StaticMemberExpression(member) => mark_read(self, &member.object),
            Expression::ComputedMemberExpression(member) => mark_read(self, &member.object),
            Expression::CallExpression(call) => mark_spreads(self, &call.arguments),
            Expression::NewExpression(new) => mark_spreads(self, &new.arguments),
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    if let ArrayExpressionElement::SpreadElement(spread) = element {
                        mark_read(self, &spread.argument);
                    }
                }
            }
            Expression::ObjectExpression(object) => {
                for property in &object.properties {
                    if let ObjectPropertyKind::SpreadProperty(spread) = property {
                        mark_read(self, &spread.argument);
                    }
                }
            }
            _ => {}
        }
    }

    pub(in super::super::super) fn mark_for_of_registry(&mut self, right: &Expression<'_>) {
        if self.collect_value_flow {
            mark_read(self, right);
        }
    }

    pub(in super::super::super) fn mark_member_object_registry(&mut self, object: &Expression<'_>) {
        if self.collect_value_flow {
            mark_read(self, object);
        }
    }

    pub(in super::super::super) fn mark_argument_spreads(&mut self, arguments: &[Argument<'_>]) {
        if self.collect_value_flow {
            mark_spreads(self, arguments);
        }
    }

    pub(in super::super::super) fn note_closed_registry_occurrence(
        &mut self,
        identifier: &IdentifierReference<'_>,
    ) {
        if !self.collect_value_flow || self.closed_registry_reads.contains(&identifier.span.start) {
            return;
        }
        self.record_registry_mutation(identifier.name.as_str());
    }

    pub(in super::super::super) fn note_exported_registry(&mut self, name: &str) {
        if !self.collect_value_flow {
            return;
        }
        self.record_registry_mutation(name);
    }
}

fn mark_spreads(collector: &mut super::super::super::ImportCollector, arguments: &[Argument<'_>]) {
    for argument in arguments {
        if let Argument::SpreadElement(spread) = argument {
            mark_read(collector, &spread.argument);
        }
    }
}

fn mark_read(collector: &mut super::super::super::ImportCollector, expr: &Expression<'_>) {
    let Expression::Identifier(identifier) = crate::codebase::ts_source::unwrap_ts_wrappers(expr)
    else {
        return;
    };
    collector
        .closed_registry_reads
        .insert(identifier.span.start);
}
