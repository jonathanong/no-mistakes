/// Records the values a namespace body declares, `(namespace path, name)`: a
/// class, function, enum, namespace, variable, or `import Alias = ...` alias,
/// `export`ed or not and wherever it sits in the body. The body is a scope of
/// its own, so each of them hides an import of the same name for every
/// construction written there, including one above a `const` or beside a class,
/// which the walk's own bindings do not cover. Types are erased and bind no
/// value.
fn scan_namespace_locals(
    collector: &mut ImportCollector,
    statements: &[Statement<'_>],
    path: &str,
) {
    let state = &mut collector.namespace;
    for statement in statements {
        if let Statement::TSNamespaceDeclaration(inner) = statement {
            let inner = format!("{path}.{}", inner.id.name);
            state.private_namespaces.insert(inner);
        }
        let exported = matches!(statement, Statement::ExportDeclaration(_));
        for name in statement_value_names(statement) {
            let local = (path.to_string(), name);
            if exported {
                state.exported_locals.insert(local.clone());
            }
            state.facts.locals.push(local);
        }
    }
}

/// A dotted `namespace A.B` exports `B` from `A`, so `B` is a value of `A`'s
/// body.
fn add_dotted_local(collector: &mut ImportCollector, path: &str, name: &str) {
    let local = (path.to_string(), name.to_string());
    collector.namespace.exported_locals.insert(local.clone());
    collector.namespace.facts.locals.push(local);
}

fn statement_value_names(statement: &Statement<'_>) -> Vec<String> {
    match statement {
        Statement::ClassDeclaration(class) => class_value_names(class),
        Statement::FunctionDeclaration(function) => function_name(function).into_iter().collect(),
        Statement::TSEnumDeclaration(declaration) => vec![declaration.id.name.to_string()],
        Statement::TSNamespaceDeclaration(declaration) => vec![declaration.id.name.to_string()],
        Statement::VariableDeclaration(declaration) => variable_value_names(declaration),
        Statement::TSImportEqualsDeclaration(declaration) => vec![declaration.id.name.to_string()],
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::ClassDeclaration(class) => class_value_names(class),
            Declaration::FunctionDeclaration(function) => {
                function_name(function).into_iter().collect()
            }
            Declaration::TSEnumDeclaration(declaration) => vec![declaration.id.name.to_string()],
            Declaration::TSNamespaceDeclaration(declaration) => {
                vec![declaration.id.name.to_string()]
            }
            Declaration::VariableDeclaration(declaration) => variable_value_names(declaration),
            Declaration::TSImportEqualsDeclaration(declaration) => {
                vec![declaration.id.name.to_string()]
            }
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

fn class_value_names(class: &Class<'_>) -> Vec<String> {
    class.id.iter().map(|id| id.name.to_string()).collect()
}

fn variable_value_names(declaration: &VariableDeclaration<'_>) -> Vec<String> {
    let declarators = declaration.declarations.iter();
    declarators
        .flat_map(|declarator| binding_names(&declarator.id))
        .collect()
}

/// Drops each local of a namespace declared in more than one block that not
/// every block shares. A block's unexported declarations are private to it, but
/// the facts name a namespace by path alone, so the blocks cannot be told apart
/// and their constructions keep the import. An exported one is shared when the
/// blocks are one namespace: top-level blocks of a name always are, and nested
/// ones are when each is exported and their parent's blocks are one namespace
/// too. A path with one block keeps its locals, because only constructions in
/// that block read them. Runs before `declared` is deduplicated, while it still
/// lists every block.
fn retain_unmerged_locals(state: &mut NamespaceState) {
    let NamespaceState {
        facts,
        exported_locals,
        private_namespaces,
        ..
    } = state;
    let mut blocks = facts.declared.clone();
    blocks.sort();
    let merged: FxHashSet<&str> = blocks
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].as_str())
        .collect();
    let one_namespace = |path: &str| {
        std::iter::successors(Some(path), |&path| {
            path.rsplit_once('.').map(|(parent, _)| parent)
        })
        .take_while(|path| merged.contains(path))
        .all(|path| !private_namespaces.contains(path))
    };
    facts.locals.retain(|local| {
        let path = local.0.as_str();
        !merged.contains(path) || exported_locals.contains(local) && one_namespace(path)
    });
    facts.locals.sort();
    facts.locals.dedup();
}
