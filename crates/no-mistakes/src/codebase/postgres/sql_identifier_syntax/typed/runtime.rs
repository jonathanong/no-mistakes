use super::*;
use oxc_ast::ast::{Class, ClassElement, Function, MethodDefinitionKind, StaticMemberExpression};

mod parameters;
mod producers;

// A field annotation is only a possible executor. Keep runtime receiver reads,
// scoped to its owning class, without promoting any SQL argument to a literal.
pub(super) fn collect(program: &Program<'_>, possible: &FxHashSet<Span>) -> FxHashSet<Span> {
    if possible.is_empty() {
        return FxHashSet::default();
    }
    let mut visitor = Runtime {
        possible,
        fields: FxHashMap::default(),
        methods: FxHashMap::default(),
        enabled: None,
        uses: FxHashSet::default(),
    };
    visitor.visit_program(program);
    visitor.uses.extend(producers::collect(program, possible));
    visitor.uses
}
struct Runtime<'s> {
    possible: &'s FxHashSet<Span>,
    fields: FxHashMap<(bool, bool), FxHashSet<String>>,
    methods: FxHashMap<u32, bool>,
    enabled: Option<bool>,
    uses: FxHashSet<Span>,
}
impl<'a> Visit<'a> for Runtime<'_> {
    fn visit_class(&mut self, class: &Class<'a>) {
        // Heritage, decorators, and computed keys evaluate with enclosing this.
        self.visit_decorators(&class.decorators);
        if let Some(heritage) = &class.heritage {
            self.visit_expression(&heritage.expression);
        }
        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    self.visit_decorators(&method.decorators);
                    self.visit_property_key(&method.key);
                }
                ClassElement::PropertyDefinition(property) => {
                    self.visit_decorators(&property.decorators);
                    self.visit_property_key(&property.key);
                }
                ClassElement::AccessorProperty(property) => {
                    self.visit_decorators(&property.decorators);
                    self.visit_property_key(&property.key);
                }
                _ => {}
            }
        }
        let fields = std::mem::take(&mut self.fields);
        let methods = std::mem::take(&mut self.methods);
        let enabled = self.enabled;
        for element in &class.body.body {
            match element {
                ClassElement::PropertyDefinition(property) => {
                    if let (Some(name), Some(annotation)) =
                        (field_name(&property.key), &property.type_annotation)
                    {
                        if possible::contains(self.possible, &annotation.type_annotation) {
                            self.fields
                                .entry((property.r#static, name.0))
                                .or_default()
                                .insert(name.1);
                        }
                    }
                }
                ClassElement::AccessorProperty(property) => {
                    if let (Some(name), Some(annotation)) =
                        (field_name(&property.key), &property.type_annotation)
                    {
                        if possible::contains(self.possible, &annotation.type_annotation) {
                            self.fields
                                .entry((property.r#static, name.0))
                                .or_default()
                                .insert(name.1);
                        }
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    self.methods
                        .insert(method.value.span.start, method.r#static);
                    if method.kind == MethodDefinitionKind::Constructor {
                        self.parameter_properties(&method.value.params);
                    }
                }
                _ => {}
            }
        }
        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) => {
                    self.visit_function(&method.value, oxc_syntax::scope::ScopeFlags::Function);
                }
                ClassElement::PropertyDefinition(property) => {
                    self.enabled = Some(property.r#static);
                    if let Some(value) = &property.value {
                        self.visit_expression(value);
                    }
                }
                ClassElement::AccessorProperty(property) => {
                    self.enabled = Some(property.r#static);
                    if let Some(value) = &property.value {
                        self.visit_expression(value);
                    }
                }
                ClassElement::StaticBlock(block) => self.visit_static_block(block),
                _ => {}
            }
        }
        self.fields = fields;
        self.methods = methods;
        self.enabled = enabled;
    }
    fn visit_function(&mut self, function: &Function<'a>, flags: oxc_syntax::scope::ScopeFlags) {
        let enabled = self.enabled;
        self.enabled = self.methods.get(&function.span.start).copied();
        walk::walk_function(self, function, flags);
        self.enabled = enabled;
    }
    fn visit_static_block(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        let enabled = self.enabled;
        self.enabled = Some(true);
        walk::walk_static_block(self, block);
        self.enabled = enabled;
    }
    fn visit_ts_type(&mut self, _: &TSType<'a>) {}
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
        if let Some(mode) = self.enabled {
            if matches!(unwrap_ts_wrappers(object), Expression::ThisExpression(_))
                && self
                    .fields
                    .get(&(mode, private))
                    .is_some_and(|names| names.contains(name))
            {
                self.uses.insert(span(at));
            }
        }
    }
}
