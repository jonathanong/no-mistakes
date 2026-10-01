/// Records the values a namespace body declares, `(namespace path, name)`: a
/// class, function, enum, namespace, or variable, `export`ed or not and wherever
/// it sits in the body. The body is a scope of its own, so each of them hides an
/// import of the same name for every construction written there, including one
/// above a `const` or beside a class, which the walk's own bindings do not
/// cover. Types are erased and bind no value.
fn scan_namespace_locals(
    collector: &mut ImportCollector,
    statements: &[Statement<'_>],
    path: &str,
) {
    let locals = &mut collector.namespace.facts.locals;
    for statement in statements {
        for name in statement_value_names(statement) {
            locals.push((path.to_string(), name));
        }
    }
}

fn statement_value_names(statement: &Statement<'_>) -> Vec<String> {
    match statement {
        Statement::ClassDeclaration(class) => class_value_names(class),
        Statement::FunctionDeclaration(function) => function_name(function).into_iter().collect(),
        Statement::TSEnumDeclaration(declaration) => vec![declaration.id.name.to_string()],
        Statement::TSNamespaceDeclaration(declaration) => vec![declaration.id.name.to_string()],
        Statement::VariableDeclaration(declaration) => variable_value_names(declaration),
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

/// Drops the locals of every namespace declared in more than one block, and of
/// the namespaces inside one. A block's unexported declarations are private to
/// it, but the facts name a namespace by path alone, so a merged namespace's
/// blocks cannot be told apart and its constructions keep the import. Runs
/// before `declared` is deduplicated, while it still lists every block.
fn retain_unmerged_locals(facts: &mut NamespaceFacts) {
    let mut blocks = facts.declared.clone();
    blocks.sort();
    let merged: FxHashSet<&str> = blocks
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].as_str())
        .collect();
    facts.locals.retain(|(path, _)| {
        std::iter::successors(Some(path.as_str()), |&path| {
            path.rsplit_once('.').map(|(parent, _)| parent)
        })
        .all(|path| !merged.contains(path))
    });
    facts.locals.sort();
    facts.locals.dedup();
}
