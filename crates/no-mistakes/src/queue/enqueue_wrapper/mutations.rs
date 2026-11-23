use crate::codebase::ts_source::unwrap_ts_wrappers as unwrap;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Expression, Program, SimpleAssignmentTarget, UnaryExpression, UnaryOperator};
use oxc_ast_visit::{walk, Visit};
use std::collections::HashSet;

#[derive(Default)]
pub(in crate::queue) struct Mutations {
    unknown: bool,
    objects: HashSet<u32>,
    imports: HashSet<String>,
}

impl Mutations {
    pub(in crate::queue) fn collect<'a>(
        program: &'a Program<'a>,
        bindings: &ObjectBindings<'a>,
        enabled: bool,
    ) -> Self {
        let mut collector = Collector {
            bindings,
            mutations: Self::default(),
        };
        if enabled {
            collector.visit_program(program);
        }
        collector.mutations
    }

    pub(super) fn stable<'a>(
        &self,
        expr: &'a Expression<'a>,
        bindings: &ObjectBindings<'a>,
    ) -> bool {
        let mut origins = Origins::default();
        origins.follow(expr, bindings, 0);
        !self.unknown
            && !origins.unknown
            && origins.objects.is_disjoint(&self.objects)
            && origins.imports.is_disjoint(&self.imports)
    }
}

#[derive(Default)]
struct Origins {
    objects: HashSet<u32>,
    imports: HashSet<String>,
    unknown: bool,
}

impl Origins {
    fn follow<'a>(
        &mut self,
        expr: &'a Expression<'a>,
        bindings: &ObjectBindings<'a>,
        depth: usize,
    ) {
        if depth == 64 {
            self.unknown = true;
            return;
        }
        match unwrap(expr) {
            Expression::Identifier(id) => {
                if let Some(binding) = bindings.lookup(id.name.as_str(), id.span.start) {
                    if let Some(init) = binding.init {
                        self.follow(init, bindings, depth + 1);
                    }
                } else {
                    self.imports.insert(id.name.to_string());
                }
            }
            Expression::StaticMemberExpression(member) => {
                self.follow(&member.object, bindings, depth + 1);
            }
            Expression::ComputedMemberExpression(member) => {
                self.follow(&member.object, bindings, depth + 1);
            }
            Expression::ObjectExpression(object) => {
                self.objects.insert(object.span.start);
            }
            _ => {}
        }
    }
}

struct Collector<'b, 'a> {
    bindings: &'b ObjectBindings<'a>,
    mutations: Mutations,
}

impl<'a> Collector<'_, 'a> {
    fn write(&mut self, object: &'a Expression<'a>) {
        let mut origins = Origins::default();
        origins.follow(object, self.bindings, 0);
        self.mutations.unknown |= origins.unknown;
        self.mutations.objects.extend(origins.objects);
        self.mutations.imports.extend(origins.imports);
    }
}

impl<'a> Visit<'a> for Collector<'_, 'a> {
    fn visit_simple_assignment_target(&mut self, target: &SimpleAssignmentTarget<'a>) {
        match self.alloc(target) {
            SimpleAssignmentTarget::StaticMemberExpression(member) => self.write(&member.object),
            SimpleAssignmentTarget::ComputedMemberExpression(member) => self.write(&member.object),
            SimpleAssignmentTarget::PrivateFieldExpression(member) => self.write(&member.object),
            _ => {}
        }
        walk::walk_simple_assignment_target(self, target);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        if unary.operator == UnaryOperator::Delete {
            match unwrap(&self.alloc(unary).argument) {
                Expression::StaticMemberExpression(member) => self.write(&member.object),
                Expression::ComputedMemberExpression(member) => self.write(&member.object),
                Expression::PrivateFieldExpression(member) => self.write(&member.object),
                _ => {}
            }
        }
        walk::walk_unary_expression(self, unary);
    }
}
