impl ImportCollector {
    /// Marks the identifier that heads `expression` as a use the graph
    /// resolves, so it is not counted as the namespace escaping. The head is
    /// found the way `simple_callee_name` reads the callee, and the marks go in
    /// before the walk reaches the identifier.
    fn mark_namespace_head(&mut self, expression: &Expression<'_>) {
        if self.namespace.names.is_empty() {
            return;
        }
        if let Some(offset) = namespace_head(expression) {
            self.namespace.benign_heads.insert(offset);
        }
    }

    /// `export { Errors }` names a namespace the file exports; the graph reads
    /// that from the root's export list, so it is not an escape.
    fn mark_export_clause_locals(&mut self, export: &ExportNamedDeclaration<'_>) {
        if self.namespace.names.is_empty() {
            return;
        }
        for specifier in &export.specifiers {
            if let ModuleExportName::IdentifierReference(identifier) = &specifier.local {
                self.namespace.benign_heads.insert(identifier.span.start);
            }
        }
    }

    fn record_namespace_site(&mut self, caller_id: Option<CallableId>, offset: u32) {
        if let Some(namespace) = self.namespace.stack.last().cloned() {
            let site = NamespaceSite {
                caller_id,
                offset,
                namespace,
            };
            self.namespace.facts.sites.push(site);
        }
    }
}

/// The source offset of the identifier a callee starts from, for exactly the
/// callees `simple_callee_name` names.
fn namespace_head(expression: &Expression<'_>) -> Option<u32> {
    match crate::codebase::ts_source::unwrap_ts_wrappers(expression) {
        Expression::Identifier(identifier) => Some(identifier.span.start),
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().and_then(namespace_head)
        }
        Expression::StaticMemberExpression(member) => {
            match crate::codebase::ts_source::unwrap_ts_wrappers(&member.object) {
                Expression::Identifier(object) => Some(object.span.start),
                Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_) => {
                    namespace_head(&member.object)
                }
                _ => None,
            }
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::Identifier(object) =
                crate::codebase::ts_source::unwrap_ts_wrappers(&member.object)
            else {
                return None;
            };
            matches!(
                crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression),
                Expression::StringLiteral(_)
            )
            .then_some(object.span.start)
        }
        _ => None,
    }
}

fn visit_identifier_reference_with_namespaces(
    collector: &mut ImportCollector,
    identifier: &IdentifierReference<'_>,
) {
    collector.note_namespace_value_use(identifier);
    collector.push_value_symbol_reference(identifier.name.to_string());
    walk::walk_identifier_reference(collector, identifier);
}

fn visit_ts_namespace_declaration_with_path<'a>(
    collector: &mut ImportCollector,
    namespace: &TSNamespaceDeclaration<'a>,
) {
    let name = namespace.id.name.as_str();
    let path = match collector.namespace.stack.last() {
        Some(parent) => format!("{parent}.{name}"),
        None => name.to_string(),
    };
    collector.namespace.stack.push(path);
    walk::walk_ts_namespace_declaration(collector, namespace);
    collector.namespace.stack.pop();
}

/// Every `namespace`, dotted `namespace A.B`, `declare module 'x'` and
/// `declare global` body is a module block, so one hook covers them all. The
/// body is a function-like scope: its declarations, `var` included, stay out of
/// the enclosing scope, so a local named like an import does not shadow it.
fn visit_ts_module_block_with_depth<'a>(
    collector: &mut ImportCollector,
    block: &TSModuleBlock<'a>,
) {
    collector.module_block_depth += 1;
    let pushed = collector.push_lexical_scope();
    if pushed {
        let var_scope = collector.local_stack.len() - 1;
        collector.var_scope_stack.push(var_scope);
        predeclare_hoisted_var_bindings(collector, &block.body);
    }
    walk::walk_ts_module_block(collector, block);
    if pushed {
        collector.var_scope_stack.pop();
    }
    collector.pop_lexical_scope(pushed);
    collector.module_block_depth -= 1;
}

/// `value instanceof Errors.Base` reads the namespace only to name a class.
fn visit_binary_expression_with_namespaces<'a>(
    collector: &mut ImportCollector,
    binary: &BinaryExpression<'a>,
) {
    if binary.operator == BinaryOperator::Instanceof {
        collector.mark_namespace_head(&binary.right);
    }
    walk::walk_binary_expression(collector, binary);
}

/// What the namespace facts need from a named class that has a base.
fn record_class_namespace_facts(
    collector: &mut ImportCollector,
    class_id: CallableId,
    class: &Class<'_>,
) {
    let tracked = collector.namespace.member_ids.contains(&class_id);
    if class.declare || (collector.module_block_depth > 0 && !tracked) {
        collector
            .namespace
            .facts
            .unreported_class_ids
            .push(class_id);
    }
    collector.record_namespace_site(Some(class_id), 0);
    if let Some(base) = class.heritage_expression() {
        collector.mark_namespace_head(base);
    }
}

/// The finished facts: the walk's uses and sites join the scan's declarations.
fn finish_namespace_facts(state: NamespaceState) -> NamespaceFacts {
    let mut facts = state.facts;
    facts.declared.sort();
    facts.declared.dedup();
    facts.members.sort_by_key(|member| member.id);
    facts.roots.sort_by(|a, b| a.name.cmp(&b.name));
    facts.value_uses = state.value_uses.into_iter().collect();
    facts.value_uses.sort();
    facts.member_uses = state.member_uses.into_iter().collect();
    facts.member_uses.sort();
    facts
        .sites
        .sort_by_key(|site| (site.caller_id, site.offset));
    facts.opaque_specifiers.sort();
    facts.unreported_class_ids.sort();
    facts
}
