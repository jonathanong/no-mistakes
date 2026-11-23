use crate::codebase::ts_source::unwrap_ts_wrappers as unwrap;
use crate::fx::FxHashSet;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Expression, Program, SimpleAssignmentTarget, UnaryExpression, UnaryOperator};
use oxc_ast_visit::{walk, Visit};

#[derive(Default)]
pub(in crate::queue) struct Mutations {
    unknown: bool,
    objects: FxHashSet<u32>,
    locals: FxHashSet<u32>,
    imports: FxHashSet<String>,
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
        self.accepts(&origins)
    }
    pub(super) fn stable_local(&self, name: &str, at: u32, bindings: &ObjectBindings<'_>) -> bool {
        let mut origins = Origins::default();
        origins.local(name, at, bindings, 0);
        self.accepts(&origins)
    }

    pub(super) fn stable_import(&self, name: &str) -> bool {
        !self.unknown && !self.imports.contains(name)
    }

    fn accepts(&self, origins: &Origins) -> bool {
        !self.unknown
            && !origins.unknown
            && origins.locals.is_disjoint(&self.locals)
            && origins.objects.is_disjoint(&self.objects)
            && origins.imports.is_disjoint(&self.imports)
    }
}

#[path = "origins.rs"]
mod origins;
use origins::Origins;

struct Collector<'b, 'a> {
    bindings: &'b ObjectBindings<'a>,
    mutations: Mutations,
}

impl<'a> Collector<'_, 'a> {
    fn write(&mut self, object: &'a Expression<'a>) {
        let mut origins = Origins::default();
        origins.follow(object, self.bindings, 0);
        self.mutations.unknown |= origins.unknown;
        self.mutations.locals.extend(origins.locals);
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
            SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
                let mut origins = Origins::default();
                origins.local(id.name.as_str(), id.span.start, self.bindings, 0);
                self.mutations.locals.extend(origins.locals);
                self.mutations.objects.extend(origins.objects);
                self.mutations.imports.extend(origins.imports);
            }
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
