use super::super::traversal::DeclarationTraversal;
use crate::fx::FxHashMap;
use crate::queue::extract_helpers::static_string;
use crate::queue::object_bindings::ObjectBindings;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;

#[derive(Clone, Copy, Default)]
enum Member<'node, 'ast> {
    #[default]
    Absent,
    Unknown,
    Value(&'node Expression<'ast>),
}

impl<'node, 'ast> Member<'node, 'ast> {
    fn overlay(&mut self, next: Self) {
        if !matches!(next, Self::Absent) {
            *self = next;
        }
    }

    fn value(self) -> Option<&'node Expression<'ast>> {
        match self {
            Self::Value(value) => Some(value),
            Self::Absent | Self::Unknown => None,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct FlowMembers<'node, 'ast> {
    queue_name: Member<'node, 'ast>,
    job: Member<'node, 'ast>,
    children: Member<'node, 'ast>,
}

impl<'node, 'ast> FlowMembers<'node, 'ast> {
    const UNKNOWN: Self = Self {
        queue_name: Member::Unknown,
        job: Member::Unknown,
        children: Member::Unknown,
    };

    pub(super) fn queue_name(self) -> Option<String> {
        self.queue_name.value().and_then(static_string)
    }

    pub(super) fn job(self) -> Option<String> {
        self.job.value().and_then(static_string)
    }

    pub(super) fn children(self) -> Option<&'node Expression<'ast>> {
        self.children.value()
    }

    fn overlay(&mut self, next: Self) {
        self.queue_name.overlay(next.queue_name);
        self.job.overlay(next.job);
        self.children.overlay(next.children);
    }
}

pub(super) fn read<'node, 'ast>(
    object: &'node ObjectExpression<'ast>,
    bindings: &ObjectBindings<'ast>,
) -> FlowMembers<'node, 'ast> {
    Reader {
        bindings,
        memo: FxHashMap::default(),
        traversal: DeclarationTraversal::default(),
    }
    .object(object)
}

struct Reader<'bindings, 'ast> {
    bindings: &'bindings ObjectBindings<'ast>,
    memo: FxHashMap<u32, FlowMembers<'ast, 'ast>>,
    traversal: DeclarationTraversal,
}

impl<'ast> Reader<'_, 'ast> {
    fn object<'node>(&mut self, object: &'node ObjectExpression<'ast>) -> FlowMembers<'node, 'ast> {
        let mut members = FlowMembers::default();
        for property in &object.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let value = Member::Value(&property.value);
                    match crate::ast::property_key_name(&property.key).as_deref() {
                        Some("queueName") => members.queue_name = value,
                        Some("name") => members.job = value,
                        Some("children") => members.children = value,
                        Some(_) => {}
                        None => members = FlowMembers::UNKNOWN,
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => {
                    members.overlay(self.expression(&spread.argument));
                }
            }
        }
        members
    }

    fn expression<'node>(&mut self, expr: &'node Expression<'ast>) -> FlowMembers<'node, 'ast> {
        match expr.get_inner_expression() {
            Expression::ObjectExpression(object) => self.object(object),
            Expression::Identifier(id) => self.binding(id.name.as_str(), expr.span().start),
            _ => FlowMembers::UNKNOWN,
        }
    }

    fn binding(&mut self, name: &str, at: u32) -> FlowMembers<'ast, 'ast> {
        let Some(binding) = self.bindings.lookup(name, at) else {
            return FlowMembers::UNKNOWN;
        };
        let Some(value) = binding.value() else {
            return FlowMembers::UNKNOWN;
        };
        let declaration = binding.resolve_at;
        if let Some(members) = self.memo.get(&declaration) {
            return *members;
        }
        let Some(cycle_count) = self.traversal.enter(declaration) else {
            return FlowMembers::UNKNOWN;
        };
        let members = self.expression(value);
        if self.traversal.complete(declaration, cycle_count) {
            self.memo.insert(declaration, members);
        }
        members
    }
}

pub(super) fn bound_object<'a>(
    expr: &'a Expression<'a>,
    bindings: &ObjectBindings<'a>,
    depth: u8,
) -> Option<&'a ObjectExpression<'a>> {
    if depth > 8 {
        return None;
    }
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => Some(object),
        Expression::Identifier(id) => {
            let value = bindings.lookup(id.name.as_str(), id.span.start)?.value()?;
            bound_object(value, bindings, depth + 1)
        }
        _ => None,
    }
}
