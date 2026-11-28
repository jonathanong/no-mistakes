//! Calls whose receiver is provably an ECMAScript or Node built-in value.
//! The proof is per file and name based: a name is built-in only when every
//! declaration of it is. Anything else stays unproven.
mod array_escapes;
mod callback_arguments;
mod class_fields;
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
use oxc_syntax::scope::ScopeFlags;

pub use tables::GLOBAL_OBJECTS;

/// Start offsets of call expressions whose callee receiver is a provable built-in.
#[derive(Default)]
pub struct ProvenCalls {
    pub calls: FxHashSet<u32>,
    pub callbacks: FxHashMap<(u32, u32), Option<&'static [usize]>>,
    pub known_nonbuiltin: FxHashSet<(u32, u32)>,
}

pub fn collect(program: &Program<'_>) -> ProvenCalls {
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
        classes: Vec::new(),
        this_depth: 0,
        admit_next_function: false,
    };
    calls.visit_program(program);
    calls.out
}

struct Calls<'n> {
    names: &'n Names,
    out: ProvenCalls,
    classes: Vec<class_fields::ClassFrame>,
    this_depth: u32,
    admit_next_function: bool,
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
            let instance = class_fields::instance_callbacks(
                &self.classes,
                self.this_depth,
                &member.object,
                method,
            );
            if !shadowed_array_from
                && (self.names.builtin(&member.object, Some(method), 0) || instance.is_some())
            {
                self.out.calls.insert(call.span.start);
                let declared = match instance {
                    Some(positions) => positions,
                    None => self.names.callback_positions(&member.object, method),
                };
                if let Some(positions) = declared {
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

    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes.push(class_fields::ClassFrame {
            fields: class_fields::fields(self.names, class),
            allowed: Vec::new(),
        });
        walk::walk_class(self, class);
        self.classes.pop();
    }

    fn visit_method_definition(&mut self, method: &MethodDefinition<'a>) {
        walk::walk_decorators(self, &method.decorators);
        self.visit_property_key(&method.key);
        if !method.r#static {
            self.admit_next_function = true;
        }
        self.visit_function(&method.value, method.kind.scope_flags());
    }

    fn visit_property_definition(&mut self, property: &PropertyDefinition<'a>) {
        walk::walk_decorators(self, &property.decorators);
        self.visit_property_key(&property.key);
        if let Some(annotation) = &property.type_annotation {
            self.visit_ts_type_annotation(annotation);
        }
        let Some(value) = &property.value else {
            return;
        };
        if property.r#static {
            self.this_depth = self.this_depth.saturating_add(1);
            self.visit_expression(value);
            self.this_depth -= 1;
            return;
        }
        if let Some(frame) = self.classes.last_mut() {
            frame.allowed.push(self.this_depth);
        }
        self.visit_expression(value);
        if let Some(frame) = self.classes.last_mut() {
            frame.allowed.pop();
        }
    }

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        self.this_depth = self.this_depth.saturating_add(1);
        walk::walk_static_block(self, block);
        self.this_depth -= 1;
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.this_depth = self.this_depth.saturating_add(1);
        let admitted = self.admit_next_function;
        self.admit_next_function = false;
        if admitted {
            if let Some(frame) = self.classes.last_mut() {
                frame.allowed.push(self.this_depth);
            }
        }
        walk::walk_function(self, function, flags);
        if admitted {
            if let Some(frame) = self.classes.last_mut() {
                frame.allowed.pop();
            }
        }
        self.this_depth -= 1;
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "builtin_receivers_tests.rs"]
mod tests;
