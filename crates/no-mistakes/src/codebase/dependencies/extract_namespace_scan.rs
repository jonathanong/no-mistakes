/// Records the namespaces declared at the top of `program`, ahead of the walk,
/// so a class's namespace path and reachability are known when the walk reaches
/// it. The scan reads statement lists only: it never enters an expression.
fn scan_program_namespaces(collector: &mut ImportCollector, program: &Program<'_>) {
    let imports = collector.predeclared_imported_bindings.iter().cloned();
    collector.namespace.names.extend(imports);
    let mut declarations = Vec::new();
    for statement in &program.body {
        if let Some(declaration) = top_level_namespace(statement) {
            declarations.push(declaration);
        } else if let Some(specifier) = require_import_specifier(statement) {
            collector.namespace.facts.opaque_specifiers.push(specifier);
        }
    }
    if declarations.is_empty() {
        return;
    }
    let clauses = local_export_clauses(program);
    let enums = top_level_enum_names(program);
    for (namespace, keyword) in declarations {
        let name = namespace.id.name.as_str();
        let mut exports: Vec<String> = clauses
            .iter()
            .filter(|(local, _)| local == name)
            .map(|(_, exported)| exported.clone())
            .collect();
        if keyword {
            exports.push(name.to_string());
        }
        let reachable = !exports.is_empty();
        add_namespace_root(collector, name, exports, enums.contains(name));
        scan_namespace(collector, namespace, "", reachable);
    }
}

fn top_level_namespace<'p, 'a>(
    statement: &'p Statement<'a>,
) -> Option<(&'p TSNamespaceDeclaration<'a>, bool)> {
    let (namespace, keyword) = match statement {
        Statement::TSNamespaceDeclaration(namespace) => (namespace, false),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::TSNamespaceDeclaration(namespace) => (namespace, true),
            _ => return None,
        },
        _ => return None,
    };
    (!namespace.declare).then_some((namespace.as_ref(), keyword))
}

/// The module of `import x = require("...")`: the whole module becomes a value,
/// which the graph does not follow.
fn require_import_specifier(statement: &Statement<'_>) -> Option<String> {
    let declaration = match statement {
        Statement::TSImportEqualsDeclaration(declaration) => declaration,
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::TSImportEqualsDeclaration(declaration) => declaration,
            _ => return None,
        },
        _ => return None,
    };
    match &declaration.module_reference {
        TSModuleReference::ExternalModuleReference(reference) => {
            Some(reference.expression.value.to_string())
        }
        _ => None,
    }
}

/// `(local, exported)` for each value specifier of an `export { ... }` clause.
fn local_export_clauses(program: &Program<'_>) -> Vec<(String, String)> {
    let clauses = program.body.iter().filter_map(|statement| match statement {
        Statement::ExportNamedDeclaration(export) if !export.export_kind.is_type() => Some(export),
        _ => None,
    });
    clauses
        .flat_map(|export| export.specifiers.iter())
        .filter(|specifier| !specifier.export_kind.is_type())
        .filter_map(|specifier| {
            let local = module_export_name_name(&specifier.local)?;
            let exported = module_export_name_name(&specifier.exported)?;
            Some((local.to_string(), exported.to_string()))
        })
        .collect()
}

fn top_level_enum_names(program: &Program<'_>) -> FxHashSet<String> {
    let enums = program.body.iter().filter_map(|statement| match statement {
        Statement::TSEnumDeclaration(declaration) => Some(declaration),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::TSEnumDeclaration(declaration) => Some(declaration),
            _ => None,
        },
        _ => None,
    });
    enums
        .map(|declaration| declaration.id.name.to_string())
        .collect()
}

fn add_namespace_root(
    collector: &mut ImportCollector,
    name: &str,
    exports: Vec<String>,
    is_enum: bool,
) {
    let roots = &mut collector.namespace.facts.roots;
    if let Some(root) = roots.iter_mut().find(|root| root.name == name) {
        let new: Vec<_> = exports.into_iter().filter(|export| !root.exports.contains(export)).collect();
        root.exports.extend(new);
        return;
    }
    let merged = is_enum || collector.local_stack[0].contains(name);
    roots.push(NamespaceRoot {
        name: name.to_string(),
        exports,
        merged,
    });
}

/// `reachable`: the namespace is exported from its parent, all the way up to a
/// root the file exports. A dotted `namespace A.B` is exported from `A`.
fn scan_namespace(
    collector: &mut ImportCollector,
    namespace: &TSNamespaceDeclaration<'_>,
    parent: &str,
    reachable: bool,
) {
    let name = namespace.id.name.as_str();
    let path = if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}.{name}")
    };
    collector.namespace.names.insert(name.to_string());
    collector.namespace.facts.declared.push(path.clone());
    match &namespace.body {
        TSNamespaceDeclarationBody::TSNamespaceDeclaration(inner) => {
            scan_namespace(collector, inner, &path, reachable);
        }
        TSNamespaceDeclarationBody::TSModuleBlock(block) => {
            for statement in &block.body {
                scan_namespace_statement(collector, statement, &path, reachable);
            }
        }
    }
}

fn scan_namespace_statement(
    collector: &mut ImportCollector,
    statement: &Statement<'_>,
    path: &str,
    reachable: bool,
) {
    match statement {
        Statement::ClassDeclaration(class) => add_namespace_member(collector, class, path, false),
        Statement::TSNamespaceDeclaration(inner) if !inner.declare => {
            scan_namespace(collector, inner, path, false);
        }
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::ClassDeclaration(class) => {
                add_namespace_member(collector, class, path, reachable);
            }
            Declaration::TSNamespaceDeclaration(inner) if !inner.declare => {
                scan_namespace(collector, inner, path, reachable);
            }
            _ => {}
        },
        _ => {}
    }
}

fn add_namespace_member(
    collector: &mut ImportCollector,
    class: &Class<'_>,
    namespace: &str,
    exported: bool,
) {
    let Some(name) = class.id.as_ref().filter(|_| !class.declare) else {
        return;
    };
    let id = CallableId(class.span.start);
    collector.namespace.member_ids.insert(id);
    collector.namespace.facts.members.push(NamespaceMember {
        path: format!("{namespace}.{}", name.name),
        id,
        exported,
    });
}
