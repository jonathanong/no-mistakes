#[derive(Default)]
struct CreatedRequireState {
    bindings: FxHashMap<usize, FxHashSet<String>>,
    factories: FxHashSet<String>,
    namespaces: FxHashSet<String>,
}

fn note_created_require_import(collector: &mut ImportCollector, import: &ImportDeclaration<'_>) {
    if import.import_kind.is_type() {
        return;
    }
    let specifier = crate::ast::js_literal(&import.source);
    if specifier != "node:module" && specifier != "module" {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                if named.import_kind.is_type() {
                    continue;
                }
                if crate::ast::module_export_name(&named.imported) == "createRequire" {
                    collector
                        .created_requires
                        .factories
                        .insert(named.local.name.to_string());
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(defaulted) => {
                collector
                    .created_requires
                    .namespaces
                    .insert(defaulted.local.name.to_string());
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                collector
                    .created_requires
                    .namespaces
                    .insert(namespace.local.name.to_string());
            }
        }
    }
}

fn record_created_require_declaration(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'_>,
) {
    if declaration.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(local) = &declarator.id else {
            continue;
        };
        let Some(init) = declarator.init.as_ref() else {
            continue;
        };
        let Expression::CallExpression(call) = init.get_inner_expression() else {
            continue;
        };
        if call.optional || !created_require_factory(collector, call.callee.get_inner_expression())
        {
            continue;
        }
        collector
            .created_requires
            .bindings
            .entry(collector.current_lexical_scope_id())
            .or_default()
            .insert(local.name.to_string());
    }
}

fn created_require_factory(collector: &ImportCollector, callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(ident) => factory_unshadowed(
            collector,
            &collector.created_requires.factories,
            ident.name.as_str(),
        ),
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "createRequire" =>
        {
            let Expression::Identifier(ident) = member.object.get_inner_expression() else {
                return false;
            };
            factory_unshadowed(
                collector,
                &collector.created_requires.namespaces,
                ident.name.as_str(),
            )
        }
        _ => false,
    }
}

fn factory_unshadowed(collector: &ImportCollector, names: &FxHashSet<String>, name: &str) -> bool {
    names.contains(name) && collector.callee_binding_scope(name).is_none()
}

fn created_require_kind(
    collector: &ImportCollector,
    callee: &Expression<'_>,
) -> Option<ImportKind> {
    match callee.get_inner_expression() {
        Expression::Identifier(ident)
            if created_require_binding(collector, ident.name.as_str()) =>
        {
            Some(ImportKind::Require)
        }
        Expression::StaticMemberExpression(member)
            if !member.optional && member.property.name == "resolve" =>
        {
            let Expression::Identifier(ident) = member.object.get_inner_expression() else {
                return None;
            };
            created_require_binding(collector, ident.name.as_str())
                .then_some(ImportKind::RequireResolve)
        }
        _ => None,
    }
}

fn created_require_binding(collector: &ImportCollector, name: &str) -> bool {
    collector.callee_binding_scope(name).is_some_and(|scope| {
        collector
            .created_requires
            .bindings
            .get(&scope)
            .is_some_and(|names| names.contains(name))
    })
}

fn record_require_import(collector: &mut ImportCollector, call: &CallExpression<'_>) -> bool {
    let require_callee = is_require_resolve_callee(&call.callee)
        .then_some("require.resolve")
        .or_else(|| is_require_callee(&call.callee).then_some("require"));
    if let Some(callee) = require_callee.filter(|_| collector.global_require_available()) {
        collector.function_calls.push(FunctionCall {
            caller: collector.current_function(),
            caller_id: collector.current_function_id(),
            syntactic_caller: collector.current_syntactic_caller(),
            callee: callee.to_string(),
            line: import_line_at(&collector.line_starts, call.span.start as usize),
            offset: call.span.start,
            is_callback: false,
            invocation: InvocationKind::Call,
            target_identity: collector.call_target_identity(callee),
            callee_binding_scope: collector.callee_binding_scope(callee),
            static_arg: call.arguments.first().and_then(static_path_argument),
            static_cwd: None,
        });
        let kind = if callee == "require.resolve" {
            ImportKind::RequireResolve
        } else {
            ImportKind::Require
        };
        record_runtime_require_import(collector, call, kind, true);
        return true;
    }
    let Some(kind) = created_require_kind(collector, &call.callee) else {
        return false;
    };
    record_runtime_require_import(collector, call, kind, false);
    false
}
