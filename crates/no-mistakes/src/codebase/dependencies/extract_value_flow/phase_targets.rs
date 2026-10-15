use super::*;
use oxc_ast::ast::{AssignmentTargetMaybeDefault, AssignmentTargetProperty};
use phase_walk::{PhaseWalk, PhaseWrite};

impl PhaseWalk<'_> {
    pub(super) fn target(&mut self, target: &AssignmentTarget<'_>) {
        let pattern = matches!(
            target,
            AssignmentTarget::ArrayAssignmentTarget(_)
                | AssignmentTarget::ObjectAssignmentTarget(_)
        );
        if pattern {
            self.nested_pattern |= self.pattern_depth > 0;
            self.pattern_depth += 1;
        }
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
                                self.conditional_expression(init);
                            }
                            self.phase += 1;
                            self.root_write(
                                property.binding.name.as_str(),
                                property.binding.span.start,
                                true,
                            );
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
                self.write(target, None, true);
            }
        }
        if pattern {
            self.pattern_depth -= 1;
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
                self.conditional_expression(&default.init);
                self.target(&default.binding);
            } else {
                self.reference(&default.binding);
                self.conditional_expression(&default.init);
                self.write(&default.binding, None, true);
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
    pub(super) fn write(
        &mut self,
        target: &AssignmentTarget<'_>,
        _value: Option<&Expression<'_>>,
        guaranteed: bool,
    ) {
        self.phase += 1;
        for (name, path, target) in write_targets::targets(target) {
            if path.is_empty() {
                self.root_write(&name, target, guaranteed);
                continue;
            }
            self.writes.push(PhaseWrite {
                target,
                name,
                path,
                phase: self.phase,
            });
        }
    }
    pub(super) fn root_write(&mut self, name: &str, target: u32, guaranteed: bool) {
        if let Some(changed) = &mut self.changed_roots {
            changed.insert(name.to_string());
        }
        if self.collect_root_origins {
            self.root_writes.push(phase_walk::RootPhaseWrite {
                target,
                name: name.to_string(),
                phase: self.phase,
                guaranteed: guaranteed && !self.conditional,
            });
        }
    }
}
