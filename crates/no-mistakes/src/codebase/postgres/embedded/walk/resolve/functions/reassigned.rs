use super::super::for_each_bound_name;
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, Program,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

/// Names assigned anywhere in the program, e.g. `build = externalBuilder;`,
/// `({ build } = providers);`, `for (build of providers)`, or
/// `var build = externalBuilder;` reassigning a hoisted `function build() {}`
/// directly, through destructuring, through a for-in/for-of loop target, or
/// through a same-named re-declaration with an initializer. A function
/// declaration's binding is mutable, so a call to it can no longer be
/// trusted to run the originally-collected body once any assignment or
/// initialized re-declaration of that name exists anywhere —
/// `LocalFunctions::collect` drops such names outright rather than
/// resolving through a body that may not be the one that runs.
///
/// Hooking the generic `visit_assignment_target` — rather than
/// `visit_assignment_expression` specifically — is what catches the loop
/// case too: a `for (build of providers)` target reaches the same
/// `AssignmentTarget` node through `ForStatementLeft`, with no separate
/// `AssignmentExpression` in between.
#[derive(Default)]
pub(super) struct ReassignedNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> ReassignedNames<'a> {
    pub(super) fn collect(program: &Program<'a>) -> Self {
        let mut reassigned = Self::default();
        reassigned.visit_program(program);
        reassigned
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }
}

impl<'a> Visit<'a> for ReassignedNames<'a> {
    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        let names = &mut self.names;
        for_each_assigned_name(target, &mut |name| {
            names.insert(name);
        });
        walk::walk_assignment_target(self, target);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if declarator.init.is_some() {
            let names = &mut self.names;
            for_each_bound_name(&declarator.id, &mut |name| {
                names.insert(name);
            });
        }
        walk::walk_variable_declarator(self, declarator);
    }
}

/// Every name a (possibly destructuring) assignment target writes to,
/// however deeply nested — `x`, `{ a: x }`, `[x]`, `{ x = 1 }`, and any
/// rest element or combination of those. A check that only handled a bare
/// `AssignmentTargetIdentifier` would miss `({ build } = providers)`
/// reassigning a same-named top-level helper through destructuring, letting
/// `LocalFunctions::collect` keep resolving calls through its stale body.
fn for_each_assigned_name<'a>(target: &AssignmentTarget<'a>, on_name: &mut impl FnMut(&'a str)) {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => on_name(ident.name.as_str()),
        AssignmentTarget::ArrayAssignmentTarget(array) => {
            for element in array.elements.iter().flatten() {
                for_each_assigned_name_maybe_default(element, on_name);
            }
            if let Some(rest) = &array.rest {
                for_each_assigned_name(&rest.target, on_name);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object) => {
            for property in &object.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(property) => {
                        on_name(property.binding.name.as_str());
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        for_each_assigned_name_maybe_default(&property.binding, on_name);
                    }
                }
            }
            if let Some(rest) = &object.rest {
                for_each_assigned_name(&rest.target, on_name);
            }
        }
        _ => {}
    }
}

fn for_each_assigned_name_maybe_default<'a>(
    target: &AssignmentTargetMaybeDefault<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    if let Some(target) = target.as_assignment_target() {
        for_each_assigned_name(target, on_name);
        return;
    }
    if let AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) = target {
        for_each_assigned_name(&with_default.binding, on_name);
    }
}
