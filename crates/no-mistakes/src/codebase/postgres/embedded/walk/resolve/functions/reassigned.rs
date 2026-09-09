use super::super::for_each_bound_name;
use super::shadows::is_function_shaped;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentTarget, AssignmentTargetMaybeDefault,
    AssignmentTargetProperty, ForStatementLeft, FormalParameters, Function, Program,
    SimpleAssignmentTarget, UpdateExpression, VariableDeclaration, VariableDeclarationKind,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

/// Names assigned anywhere in the program, e.g. `build = externalBuilder;`,
/// `({ build } = providers);`, `for (build of providers)`,
/// `for (var build of providers) {}`, or `var build = externalBuilder;`
/// reassigning a hoisted `function build() {}` directly, through
/// destructuring, through a for-in/for-of loop target (plain, or declared
/// with `var`), or through a same-named re-declaration with an initializer.
/// A `let`/`const` loop target is lexically scoped to the loop and is
/// deliberately excluded — see the `visit_for_statement_left` doc comment
/// below. A function
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
/// (`function`/arrow expression, matching [`shadows::is_function_shaped`])
/// declared `const` — see [`record_declarator_reassignment`] for why the
/// exemption is narrowed to that one declaration kind.
#[derive(Default)]
pub(super) struct ReassignedNames<'a> {
    names: HashSet<&'a str>,
    param_stack: Vec<HashSet<&'a str>>,
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

    fn records_name(&self, name: &str) -> bool {
        !self
            .param_stack
            .iter()
            .rev()
            .any(|params| params.contains(name))
    }

    fn push_params(&mut self, params: &FormalParameters<'a>) {
        let mut names = HashSet::new();
        for item in &params.items {
            for_each_bound_name(&item.pattern, &mut |name| {
                names.insert(name);
            });
        }
        if let Some(rest) = &params.rest {
            for_each_bound_name(&rest.rest.argument, &mut |name| {
                names.insert(name);
            });
        }
        self.param_stack.push(names);
    }
}

impl<'a> Visit<'a> for ReassignedNames<'a> {
    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.push_params(&function.params);
        walk::walk_function(self, function, flags);
        self.param_stack.pop();
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.push_params(&arrow.params);
        walk::walk_arrow_function_expression(self, arrow);
        self.param_stack.pop();
    }

    fn visit_assignment_target(&mut self, target: &AssignmentTarget<'a>) {
        let mut assigned = Vec::new();
        for_each_assigned_name(target, &mut |name| assigned.push(name));
        for name in assigned {
            if self.records_name(name) {
                self.names.insert(name);
            }
        }
        walk::walk_assignment_target(self, target);
    }

    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        let mut rebound = Vec::new();
        for declarator in &declaration.declarations {
            record_declarator_reassignment(declaration.kind, declarator, &mut |name| {
                rebound.push(name);
            });
        }
        for name in rebound {
            if self.records_name(name) {
                self.names.insert(name);
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }

    /// A declaration-form loop target (`for (const build of providers) {}`)
    /// binds a fresh value from the iterable on every iteration, exactly
    /// like the non-declaration form `for (build of providers)` already
    /// caught via `visit_assignment_target` — but its `VariableDeclarator`
    /// has no initializer, so `visit_variable_declaration`'s per-declarator
    /// pass above never sees it.
    ///
    /// Only a `var` target counts as a global reassignment here: `var` is
    /// function-scoped, so it really does leak the loop's last-iterated
    /// value to a same-named top-level helper everywhere else in the
    /// function, including after the loop. A `let`/`const` target is
    /// lexically scoped to the loop itself — `for (const build of …) {}`
    /// does not rebind an outer top-level `function build() {}` for a call
    /// made after the loop, so marking it globally reassigned here would be
    /// a false positive. `ScopeVisitor::visit_for_of_statement`/
    /// `visit_for_in_statement` separately push a scope and bind the
    /// declared target there for every kind (`var` included), so a call
    /// made *inside* the loop body still resolves against the loop-scoped
    /// value via `shadowed_locally` rather than the stale top-level body —
    /// this method only governs whether the name counts as reassigned
    /// outside the loop.
    fn visit_for_statement_left(&mut self, it: &ForStatementLeft<'a>) {
        if let ForStatementLeft::VariableDeclaration(declaration) = it {
            if declaration.kind == VariableDeclarationKind::Var {
                let mut rebound = Vec::new();
                for declarator in &declaration.declarations {
                    for_each_bound_name(&declarator.id, &mut |name| rebound.push(name));
                }
                for name in rebound {
                    if self.records_name(name) {
                        self.names.insert(name);
                    }
                }
            }
        }
        walk::walk_for_statement_left(self, it);
    }

    /// `build++`/`build--` writes through `UpdateExpression.argument`, a
    /// `SimpleAssignmentTarget` — a separate node type from the
    /// `AssignmentTarget` `visit_assignment_target` above already catches,
    /// reached through its own visitor method rather than as a case of it.
    /// Only the identifier form is a same-file-helper-name rebinding; a
    /// member-expression operand (`obj.build++`) writes through a property,
    /// not a name `LocalFunctions` tracks, so it's left alone.
    fn visit_update_expression(&mut self, it: &UpdateExpression<'a>) {
        if let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &it.argument {
            if self.records_name(ident.name.as_str()) {
                self.names.insert(ident.name.as_str());
            }
        }
        walk::walk_update_expression(self, it);
    }
}

/// A declarator's own initializer is exempted from counting as a
/// reassignment when it is itself a same-file callable-helper shape
/// (`function`/arrow expression, matching [`is_function_shaped`]) **and**
/// its declaration is `const`: that combination is exactly what
/// `collect::const_resolvable` collects as a legitimate same-file helper,
/// not a rebinding of one — a literal function/arrow value can't itself be
/// "the wrong body" the way an identifier reference like `externalBuilder`
/// can.
///
/// `var`/`let` never reach `const_resolvable` — only `const` function
/// expressions are collected as helpers — so a same-shaped
/// `var build = () => …;` following an earlier `function build() {}` is a
/// real runtime rebinding to a body `LocalFunctions` never re-collects: it
/// must still count as a reassignment, or calls to `build` keep resolving
/// through the stale `function` declaration's body. A re-declaration whose
/// initializer resolves to something else (an identifier, a call, ...)
/// still counts regardless of kind, which is what keeps
/// `var build = externalBuilder;` rejected after an earlier
/// `function build() {}`.
fn record_declarator_reassignment<'a>(
    kind: VariableDeclarationKind,
    declarator: &VariableDeclarator<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    let is_collectible_helper = kind == VariableDeclarationKind::Const
        && declarator.init.as_ref().is_some_and(is_function_shaped);
    let is_reassignment = declarator.init.is_some() && !is_collectible_helper;
    if is_reassignment {
        for_each_bound_name(&declarator.id, on_name);
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
