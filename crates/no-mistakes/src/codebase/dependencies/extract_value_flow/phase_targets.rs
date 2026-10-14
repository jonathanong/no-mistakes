use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};
use phase_walk::{PhaseWalk, PhaseWrite};

impl PhaseWalk<'_> {
    pub(super) fn target(&mut self, target: &AssignmentTarget<'_>) {
        match target {
            AssignmentTarget::ArrayAssignmentTarget(array) => {
                for element in array.elements.iter().flatten() {
                    self.maybe_default(element);
                }
                if let Some(rest) = &array.rest {
                    self.target(&rest.target);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(object) => {
                for property in &object.properties {
                    match property {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                            if let Some(init) = &property.init {
                                self.expression(init);
                            }
                        }
                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            if property.computed {
                                if let Some(key) = property.name.as_expression() {
                                    self.expression(key);
                                }
                            }
                            self.maybe_default(&property.binding);
                        }
                    }
                }
                if let Some(rest) = &object.rest {
                    self.target(&rest.target);
                }
            }
            _ => {
                self.reference(target);
                self.write(target);
            }
        }
    }
    fn maybe_default(&mut self, target: &AssignmentTargetMaybeDefault<'_>) {
        if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) = target {
            // A simple target reference is evaluated before its default, while
            // a nested pattern consumes the default before evaluating targets.
            if matches!(
                default.binding,
                AssignmentTarget::ArrayAssignmentTarget(_)
                    | AssignmentTarget::ObjectAssignmentTarget(_)
            ) {
                self.expression(&default.init);
                self.target(&default.binding);
            } else {
                self.reference(&default.binding);
                self.expression(&default.init);
                self.write(&default.binding);
            }
        } else if let Some(target) = target.as_assignment_target() {
            self.target(target);
        }
    }
    pub(super) fn reference(&mut self, target: &AssignmentTarget<'_>) {
        match target {
            AssignmentTarget::StaticMemberExpression(member) => self.expression(&member.object),
            AssignmentTarget::ComputedMemberExpression(member) => {
                self.expression(&member.object);
                self.expression(&member.expression);
            }
            _ => {}
        }
    }
    pub(super) fn write(&mut self, target: &AssignmentTarget<'_>) {
        self.phase += 1;
        for (name, path) in write_targets::targets(target)
            .into_iter()
            .filter(|(_, path)| !path.is_empty())
        {
            let base = name.split('.').next().expect("receiver write");
            let scope = self.collector.callee_binding_scope(base).or_else(|| {
                (self.collector.imported_bindings.contains(base)
                    || self.collector.predeclared_imported_bindings.contains(base))
                .then_some(0)
            });
            if let Some(scope) = scope {
                self.writes.push(PhaseWrite {
                    scope,
                    name,
                    path,
                    phase: self.phase,
                });
            }
        }
    }
}
