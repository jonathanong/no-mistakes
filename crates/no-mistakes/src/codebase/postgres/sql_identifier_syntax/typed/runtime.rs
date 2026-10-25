use super::*;
use oxc_ast::ast::{
    CallExpression, Class, Function, ObjectExpression, StaticMemberExpression, VariableDeclarator,
};

mod classes;
mod objects;
mod parameters;
mod producers;

// A field annotation is only a possible executor. Keep runtime receiver reads,
// scoped to its owning class, without promoting any SQL argument to a literal.
pub(super) fn collect(
    program: &Program<'_>,
    possible: &FxHashSet<Span>,
    bindings: &FxHashMap<Span, usize>,
) -> (FxHashSet<Span>, FxHashSet<usize>, FxHashSet<Span>) {
    if possible.is_empty() {
        return (
            FxHashSet::default(),
            FxHashSet::default(),
            FxHashSet::default(),
        );
    }
    let mut visitor = Runtime {
        possible,
        bindings,
        fields: FxHashMap::default(),
        methods: FxHashMap::default(),
        enabled: None,
        object_fields: FxHashMap::default(),
        this_fields: None,
        reads: Vec::new(),
        uses: FxHashSet::default(),
    };
    visitor.visit_program(program);
    let (producer_uses, produced, transports) = producers::collect(program, possible, bindings);
    visitor.uses.extend(producer_uses);
    (visitor.uses, produced, transports)
}
struct Runtime<'s> {
    possible: &'s FxHashSet<Span>,
    bindings: &'s FxHashMap<Span, usize>,
    fields: FxHashMap<(bool, bool), FxHashSet<String>>,
    methods: FxHashMap<u32, bool>,
    enabled: Option<bool>,
    object_fields: FxHashMap<usize, FxHashSet<String>>,
    this_fields: Option<FxHashSet<String>>,
    reads: Vec<Span>,
    uses: FxHashSet<Span>,
}
impl<'a> Visit<'a> for Runtime<'_> {
    fn visit_class(&mut self, class: &Class<'a>) {
        classes::visit(self, class);
    }
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let enabled = self.enabled;
        let this_fields = self.this_fields.take();
        self.enabled = self.methods.get(&function.span.start).copied();
        walk::walk_function(self, function, flags);
        self.enabled = enabled;
        self.this_fields = this_fields;
    }
    fn visit_static_block(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        let enabled = self.enabled;
        self.enabled = Some(true);
        walk::walk_static_block(self, block);
        self.enabled = enabled;
    }
    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        objects::call(self, call);
    }
    fn visit_variable_declarator(&mut self, variable: &VariableDeclarator<'a>) {
        objects::variable(self, variable);
    }
    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        objects::visit(self, object);
    }
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        self.record(
            &member.object,
            false,
            member.property.name.as_str(),
            member.span,
        );
        walk::walk_static_member_expression(self, member);
    }
    fn visit_computed_member_expression(
        &mut self,
        member: &oxc_ast::ast::ComputedMemberExpression<'a>,
    ) {
        if let Expression::StringLiteral(key) = unwrap_ts_wrappers(&member.expression) {
            self.record(&member.object, false, key.value.as_str(), member.span);
        }
        walk::walk_computed_member_expression(self, member);
    }
    fn visit_private_field_expression(
        &mut self,
        member: &oxc_ast::ast::PrivateFieldExpression<'a>,
    ) {
        self.record(
            &member.object,
            true,
            member.field.name.as_str(),
            member.span,
        );
        walk::walk_private_field_expression(self, member);
    }
}

fn field_name(key: &oxc_ast::ast::PropertyKey<'_>) -> Option<(bool, String)> {
    match key {
        oxc_ast::ast::PropertyKey::PrivateIdentifier(name) => Some((true, name.name.to_string())),
        _ => key.static_name().map(|name| (false, name.to_string())),
    }
}
impl Runtime<'_> {
    fn record(&mut self, object: &Expression<'_>, private: bool, name: &str, at: oxc_span::Span) {
        let object = unwrap_ts_wrappers(object);
        let this_field = matches!(object, Expression::ThisExpression(_))
            && (self
                .this_fields
                .as_ref()
                .is_some_and(|fields| fields.contains(name))
                || self.enabled.is_some_and(|mode| {
                    self.fields
                        .get(&(mode, private))
                        .is_some_and(|names| names.contains(name))
                }));
        let object_field = if let Expression::Identifier(id) = object {
            self.bindings
                .get(&span(id.span))
                .and_then(|id| self.object_fields.get(id))
                .is_some_and(|fields| fields.contains(name))
        } else {
            false
        };
        if this_field || object_field {
            self.reads.push(span(at));
        }
    }
}
