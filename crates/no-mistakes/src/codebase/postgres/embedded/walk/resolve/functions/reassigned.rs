use super::super::for_each_bound_name;
use super::shadows::is_function_shaped;
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetProperty, ForStatementLeft,
    Program, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

/// Names assigned anywhere in the program, e.g. `build = externalBuilder;`,
/// `({ build } = providers);`, `for (build of providers)`,
/// `for (const build of providers) {}`, or `var build = externalBuilder;`
/// reassigning a hoisted `function build() {}` directly, through
/// destructuring, through a for-in/for-of loop target (declared or plain),
/// or through a same-named re-declaration with an initializer. A function
/// declaration's binding is mutable, so a call to it can no longer be
/// trusted to run the originally-collected body once any assignment or
/// initialized re-declaration of that name exists anywhere —
/// `LocalFunctions::collect` drops such names outright rather than
/// resolving through a body that may not be the one that runs.
///
/// Hooking the generic `visit_assignment_target` — rather than
/// `visit_assignment_expression` specifically — is what catches the
/// non-declaration loop case too: a `for (build of providers)` target
/// reaches the same `AssignmentTarget` node through `ForStatementLeft`, with
/// no separate `AssignmentExpression` in between. The declaration form,
/// `for (const build of providers) {}`, takes a different path entirely —
/// its `VariableDeclarator` has no initializer (the iterable lives on the
/// `ForOfStatement`/`ForInStatement`, not the declarator), so it needs its
/// own `visit_for_statement_left` hook rather than falling out of either the
/// assignment-target or declarator-with-initializer checks below.
///
/// A declarator's own initializer is exempted from counting as a
/// reassignment when it is itself a same-file callable-helper shape
/// (`function`/arrow expression, matching [`shadows::is_function_shaped`]):
/// that shape is exactly what defines a const-bound helper in the first
/// place (see `collect::const_resolvable`), not a rebinding of one — a
/// literal function/arrow value can't itself be "the wrong body" the way an
/// identifier reference like `externalBuilder` can. A re-declaration whose
/// initializer resolves to something else (an identifier, a call, ...)
/// still counts, which is what keeps `var build = externalBuilder;`
/// rejected after an earlier `function build() {}`.
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
        let is_reassignment = declarator
            .init
            .as_ref()
            .is_some_and(|init| !is_function_shaped(init));
        if is_reassignment {
            let names = &mut self.names;
            for_each_bound_name(&declarator.id, &mut |name| {
                names.insert(name);
            });
        }
        walk::walk_variable_declarator(self, declarator);
    }

    /// A declaration-form loop target (`for (const build of providers) {}`)
    /// binds a fresh value from the iterable on every iteration, exactly
    /// like the non-declaration form `for (build of providers)` already
    /// caught via `visit_assignment_target` — but its `VariableDeclarator`
    /// has no initializer, so `visit_variable_declarator` above never sees
    /// it. Mark every name it binds as reassigned unconditionally: unlike a
    /// top-level helper declaration, there is no legitimate "this declarator
    /// literally is the helper" reading here — the iterable, not this
    /// declarator, decides what the name is bound to each time.
    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        if let ForStatementLeft::VariableDeclaration(declaration) = it {
            let names = &mut self.names;
            for declarator in &declaration.declarations {
                for_each_bound_name(&declarator.id, &mut |name| {
                    names.insert(name);
                });
            }
        }
        walk::walk_for_statement_left(self, it);
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
