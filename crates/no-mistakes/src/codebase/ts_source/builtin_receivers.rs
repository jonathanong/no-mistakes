//! Calls whose receiver is provably an ECMAScript or Node built-in value.
//! The proof is per file and name based: a name is built-in only when every
//! declaration of it is. Anything else stays unproven.
mod array_escapes;
mod callback_arguments;
mod contextual;
mod dates;
mod decls;
mod known_wrong;
mod names;
mod proof;
mod return_kinds;
mod tables;
mod values;
use crate::fx::{FxHashMap, FxHashSet};
use names::Names;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(crate) use tables::GLOBAL_OBJECTS;

/// Start offsets of call expressions whose callee receiver is a provable built-in.
#[derive(Default)]
pub(crate) struct ProvenCalls {
    pub(crate) calls: FxHashSet<u32>,
    pub(crate) callbacks: FxHashMap<(u32, u32), Option<&'static [usize]>>,
    pub(crate) known_nonbuiltin: FxHashSet<(u32, u32)>,
}

pub(crate) fn collect(program: &Program<'_>) -> ProvenCalls {
    let mut names = Names::default();
    let called = return_kinds::called_names(program);
    // Initializer proofs may read names declared later; iterate to settle them.
    for _ in 0..3 {
        let mut decls = decls::Decls {
            prior: names,
            next: Names::default(),
            kind: VariableDeclarationKind::Var,
            hooks: false,
            called: &called,
        };
        decls.visit_program(program);
        names = decls.next;
    }
    array_escapes::invalidate(&mut names, program);
    let mut calls = Calls {
        names: &names,
        out: ProvenCalls::default(),
    };
    calls.visit_program(program);
    calls.out
}

struct Calls<'n> {
    names: &'n Names,
    out: ProvenCalls,
}

impl<'a> Visit<'a> for Calls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            let method = member.property.name.as_str();
            let key = (call.span.start, call.span.end);
            if known_wrong::receiver(self.names, &member.object, method) {
                self.out.known_nonbuiltin.insert(key);
            }
            let shadowed_array_from = method == "from"
                && matches!(&member.object, Expression::Identifier(id)
                    if id.name == "Array" && !self.names.global(&id.name));
            if !shadowed_array_from && self.names.builtin(&member.object, Some(method), 0) {
                self.out.calls.insert(call.span.start);
                if let Some(positions) = self.names.callback_positions(&member.object, method) {
                    let positions = callback_arguments::callable_positions(
                        self.names,
                        call,
                        &member.object,
                        method,
                        positions,
                    );
                    if let Some(positions) = positions {
                        self.out.callbacks.insert(key, Some(positions));
                        contextual::collect(
                            self.names,
                            call,
                            &member.object,
                            method,
                            positions,
                            &mut self.out,
                        );
                    }
                } else {
                    self.out.callbacks.insert(key, None);
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}

#[cfg(test)]
#[path = "builtin_receivers_tests.rs"]
mod tests;
