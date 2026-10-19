use super::*;
#[derive(Default)]
pub(in super::super) struct Side { instances: FxHashMap<(usize, String), CallableId>, methods: Vec<(u32, CallableId, Vec<ValueExpression>)>, escaped: FxHashMap<u32, FxHashMap<usize, Vec<ValueExpression>>>, open: FxHashMap<u32, (usize, Vec<ValueExpression>)>, reassigned: Vec<(usize, String, ValueExpression)> }
impl ImportCollector {
    pub(in super::super) fn note_instance_class(&mut self, declarator: &VariableDeclarator<'_>) {
        let Some(init) = &declarator.init else { return };
        let Expression::NewExpression(new) = crate::codebase::ts_source::unwrap_ts_wrappers(init) else { return };
        let Some(name) = simple_callee_name(&new.callee) else { return };
        let Some(class_id) = self.callable_binding_at(self.callee_binding_scope(&name).unwrap_or(0), &name) else { return };
        for binding in binding_names(&declarator.id) { self.receiver_escapes.instances.insert((self.callee_binding_scope(&binding).unwrap_or(self.current_lexical_scope_id()), binding), class_id); }
    }
    pub(in super::super) fn note_escaped_call(&mut self, call: &CallExpression<'_>) {
        self.note_class_method(call);
        let mut extras: FxHashMap<usize, Vec<ValueExpression>> = fx_map();
        let (mut open_at, mut open_vals, mut position): (Option<usize>, Vec<ValueExpression>, usize) = (None, Vec::new(), 0);
        for (raw_index, argument) in call.arguments.iter().enumerate() {
            match argument {
                Argument::SpreadElement(spread) => {
                    // A literal is walked so a conditional element or a nested spread is not one opaque value. Named arrays stay on `array_entries`.
                    if let Expression::ArrayExpression(array) = crate::codebase::ts_source::unwrap_ts_wrappers(&spread.argument) { for element in &array.elements { match element { oxc_ast::ast::ArrayExpressionElement::SpreadElement(_) => { open_at.get_or_insert(position); } oxc_ast::ast::ArrayExpressionElement::Elision(_) => { if open_at.is_none() { position += 1; } } other => { let values = other.as_expression().map_or_else(Vec::new, |expression| argument_values(self, expression)); if open_at.is_some() { open_vals.extend(values); } else { extras.entry(position).or_default().extend(values); position += 1; } } } } }
                    else { let value = self.value_expression(&spread.argument); match array_entries(self, &value) { Some(entries) if open_at.is_none() => { let mut next = position; for (slot, value) in entries { let at = position + slot; extras.entry(at).or_default().push(value); next = next.max(at + 1); } position = next; } Some(entries) => open_vals.extend(entries.into_iter().map(|(_, value)| value)), None => { open_at.get_or_insert(position); } } }
                }
                other => {
                    let values = other.as_expression().map_or_else(Vec::new, |expression| argument_values(self, expression));
                    if open_at.is_some() { open_vals.extend(values); }
                    else if values.len() != 1 || position != raw_index { if !values.is_empty() { extras.entry(position).or_default().extend(values); } position += 1; }
                    else { position += 1; }
                }
            }
        }
        if !extras.is_empty() { self.receiver_escapes.escaped.insert(call.span.start, extras); }
        if let Some(start) = open_at.filter(|_| !open_vals.is_empty()) { self.receiver_escapes.open.insert(call.span.start, (start, open_vals)); }
    }
    fn note_class_method(&mut self, call: &CallExpression<'_>) {
        let Expression::StaticMemberExpression(member) = crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee) else { return };
        let Some(class) = class_id(self, &member.object) else { return };
        let Some(method) = owner_member_id(&self.aggregate_callable_member_ids, class, member.property.name.as_str()) else { return };
        let arguments = call.arguments.iter().map(|argument| argument.as_expression().map_or(ValueExpression::Unknown, |expression| self.value_expression(expression))).collect();
        self.receiver_escapes.methods.push((call.span.start, method, arguments));
    }
    pub(in super::super) fn note_parameter_reassignment(&mut self, assignment: &AssignmentExpression<'_>) {
        if !self.collect_value_flow || !(assignment.operator.is_assign() || assignment.operator.is_logical()) { return }
        let Some(name) = super::mutable_receivers::mutable_root_name(&assignment.left) else { return };
        let Some(scope) = self.callee_binding_scope(name) else { return };
        let bare = self.value_parameter_indexes.get(&scope).and_then(|names| names.get(name)).is_some_and(|indexes| indexes.iter().any(|index| self.value_flow.parameters.get(*index).is_some_and(|parameter| parameter.projection.is_empty())));
        if bare { self.receiver_escapes.reassigned.push((scope, name.to_string(), self.value_expression(&assignment.right))); }
    }
    pub(in super::super) fn project_arguments_member(&mut self, offset: u32, path: Option<&[Option<String>]>, completed_at: u32) -> bool {
        let Some((head, tail)) = path.and_then(|path| path.split_first()) else { return false };
        let Some(index) = head.as_deref().and_then(|text| text.parse().ok()) else { return false };
        let Some(owner) = self.current_function_id() else { return false };
        let found = self.value_flow.parameters.iter().find(|parameter| parameter.owner == owner && parameter.index == index && parameter.projection.is_empty()).map(|parameter| (parameter.scope, parameter.name.clone()));
        let (scope, name) = found.unwrap_or_else(|| self.synthesize_arguments_parameter(owner, index));
        let dotted = std::iter::once(name.as_str()).chain(tail.iter().filter_map(|part| part.as_deref())).collect::<Vec<_>>().join(".");
        self.invalidate_value_write_with_path(scope, &dotted, offset, Some(tail), completed_at);
        true
    }
    fn synthesize_arguments_parameter(&mut self, owner: CallableId, index: usize) -> (usize, String) {
        let scope = self.current_lexical_scope_id();
        let name = format!("arguments[{index}]");
        let order = self.value_flow.parameters.len();
        self.value_parameter_indexes.entry(scope).or_default().entry(name.clone()).or_default().push(order);
        self.value_flow.parameters.push(ValueParameter { owner, index, name: name.clone(), scope, default: ValueExpression::Unknown, projection: Vec::new(), initialization_order: order, default_order: order, default_mutations: None, invalidated_at: None, mutation_contexts: fx_set(), type_reference: None });
        (scope, name)
    }
    pub(in super::super) fn object_assign_result(&self, call: &CallExpression<'_>) -> Option<ValueExpression> {
        if simple_callee_name(&call.callee).as_deref() != Some("Object.assign") || self.callee_binding_scope("Object").is_some() { return None; }
        let mut raw = call.arguments.iter();
        let Expression::ObjectExpression(target) = crate::codebase::ts_source::unwrap_ts_wrappers(raw.next()?.as_expression()?) else { return None };
        let ValueExpression::Object(target_fields, null_prototype) = self.object_value(target) else { return None };
        let mut fields = (*target_fields).clone();
        for source in raw {
            let Expression::ObjectExpression(object) = crate::codebase::ts_source::unwrap_ts_wrappers(source.as_expression()?) else { return None };
            let ValueExpression::Object(source_fields, _) = self.object_value(object) else { return None };
            fields.extend(source_fields.iter().map(|(key, value)| (key.clone(), value.clone())));
        }
        Some(ValueExpression::Object(std::sync::Arc::new(fields), null_prototype))
    }
}
fn class_id(collector: &ImportCollector, object: &Expression<'_>) -> Option<CallableId> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(object) {
        Expression::NewExpression(new) => simple_callee_name(&new.callee).and_then(|name| collector.callable_binding_at(collector.callee_binding_scope(&name).unwrap_or(0), &name)),
        Expression::Identifier(identifier) => collector.receiver_escapes.instances.get(&(collector.callee_binding_scope(identifier.name.as_str()).unwrap_or(0), identifier.name.to_string())).copied(),
        _ => None,
    }
}
fn argument_values(collector: &ImportCollector, expression: &Expression<'_>) -> Vec<ValueExpression> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::ConditionalExpression(conditional) => { let mut values = argument_values(collector, &conditional.consequent); values.extend(argument_values(collector, &conditional.alternate)); values }
        Expression::LogicalExpression(logical) => { let mut values = argument_values(collector, &logical.left); values.extend(argument_values(collector, &logical.right)); values }
        Expression::SequenceExpression(sequence) => sequence.expressions.last().map(|expression| argument_values(collector, expression)).unwrap_or_default(),
        other => vec![collector.value_expression(other)],
    }
}
pub(super) fn absorb(collector: &ImportCollector, slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>, values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>) {
    for call in &collector.value_flow.calls {
        if let (ValueExpression::Member(receiver, method), Some(callback)) = (&call.callee, call.arguments.first()) {
            let owner = super::parameter_call_origins::callable_owner(collector, callback);
            if matches!(method.as_str(), "forEach" | "map" | "filter" | "some" | "every" | "find" | "findIndex" | "findLast" | "findLastIndex" | "flatMap") {
                if let Some(items) = array_elements(collector, receiver) { for item in items { push_slot(values, slots, owner, 0, item); } }
            }
            if matches!(method.as_str(), "reduce" | "reduceRight") {
                let parameters = owner.and_then(|id| slots.get(&id));
                if let Some(key) = parameters.and_then(|parameters| parameters.get(&0)) {
                    if let Some(initial) = call.arguments.get(1) { values.entry(key.clone()).or_default().push(initial.clone()); }
                    else if let Some(items) = array_elements(collector, receiver).filter(|items| items.len() >= 2) { values.entry(key.clone()).or_default().extend(items); }
                }
                if let Some(key) = parameters.and_then(|parameters| parameters.get(&1)) {
                    if let Some(items) = array_elements(collector, receiver) { values.entry(key.clone()).or_default().extend(items); }
                }
            }
        }
        if let Some(parameters) = super::parameter_call_origins::callable_owner(collector, &call.callee).and_then(|id| slots.get(&id)) { apply_escaped(values, parameters, collector, call.offset); }
    }
    for (offset, method, arguments) in &collector.receiver_escapes.methods {
        if let Some(parameters) = slots.get(method) {
            for (index, argument) in arguments.iter().enumerate() { if let Some(key) = parameters.get(&index) { values.entry(key.clone()).or_default().push(argument.clone()); } }
            apply_escaped(values, parameters, collector, *offset);
        }
    }
    for (scope, name, value) in &collector.receiver_escapes.reassigned { values.entry((*scope, name.clone())).or_default().push(value.clone()); }
}
fn apply_escaped(values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>, parameters: &FxHashMap<usize, (usize, String)>, collector: &ImportCollector, offset: u32) {
    if let Some(extras) = collector.receiver_escapes.escaped.get(&offset) { for (index, extra) in extras { if let Some(key) = parameters.get(index) { values.entry(key.clone()).or_default().extend(extra.iter().cloned()); } } }
    if let Some((start, extra)) = collector.receiver_escapes.open.get(&offset) { for (index, key) in parameters { if *index >= *start { values.entry(key.clone()).or_default().extend(extra.iter().cloned()); } } }
}
fn push_slot(values: &mut FxHashMap<(usize, String), Vec<ValueExpression>>, slots: &FxHashMap<CallableId, FxHashMap<usize, (usize, String)>>, owner: Option<CallableId>, index: usize, item: ValueExpression) {
    if let Some(key) = owner.and_then(|id| slots.get(&id)).and_then(|parameters| parameters.get(&index)) { values.entry(key.clone()).or_default().push(item); }
}
pub(super) fn array_elements(collector: &ImportCollector, value: &ValueExpression) -> Option<Vec<ValueExpression>> {
    Some(array_entries(collector, value)?.into_iter().map(|(_, value)| value).collect())
}
pub(super) fn array_entries(collector: &ImportCollector, value: &ValueExpression) -> Option<Vec<(usize, ValueExpression)>> {
    let items = match value {
        ValueExpression::Array(items) => items.clone(),
        ValueExpression::Named { name, scope, .. } => {
            let indexes = collector.value_binding_indexes.get(&scope.unwrap_or(0))?.get(name)?;
            match &collector.value_flow.bindings[*indexes.last()?].value { ValueExpression::Array(items) => items.clone(), _ => return None }
        }
        _ => return None,
    };
    let mut entries = Vec::with_capacity(items.len());
    for (key, value) in items.iter() { entries.push((key.parse::<usize>().ok()?, value.clone())); }
    entries.sort_by_key(|(slot, _)| *slot);
    Some(entries)
}
pub(super) fn copied_member(collector: &mut ImportCollector, key: &(usize, String), path: &[Option<String>], at: u32) {
    let dotted = std::iter::once(key.1.as_str()).chain(path.iter().filter_map(|part| part.as_deref())).collect::<Vec<_>>().join(".");
    collector.invalidate_value_write_with_path(key.0, &dotted, at, Some(path), at);
}
pub(super) fn assign_sources(collector: &ImportCollector, arguments: &[ValueExpression], raw: &[oxc_ast::ast::Argument<'_>]) -> Vec<ValueExpression> {
    let mut sources = Vec::new();
    for (index, source) in arguments.iter().enumerate().skip(1) {
        if let Some(Argument::SpreadElement(spread)) = raw.get(index) {
            if let Some(entries) = array_entries(collector, &collector.value_expression(&spread.argument)) { sources.extend(entries.into_iter().map(|(_, value)| value)); continue; }
        }
        sources.push(source.clone());
    }
    sources
}
