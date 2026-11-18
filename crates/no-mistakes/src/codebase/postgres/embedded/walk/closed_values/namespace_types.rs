use super::*;
use crate::fx::FxHashSet;
use std::sync::Arc;

type Names = Arc<FxHashSet<String>>;

fn namespace<'s, 'a>(statement: &'s Statement<'a>) -> Option<&'s TSNamespaceDeclaration<'a>> {
    let declaration = match statement {
        Statement::ExportDeclaration(export) => Some(&export.declaration),
        _ => statement.as_declaration(),
    }?;
    match declaration {
        Declaration::TSNamespaceDeclaration(namespace) => Some(namespace),
        _ => None,
    }
}

fn namespace_block<'s, 'a>(namespace: &'s TSNamespaceDeclaration<'a>) -> &'s TSModuleBlock<'a> {
    let mut body = &namespace.body;
    loop {
        match body {
            TSNamespaceDeclarationBody::TSNamespaceDeclaration(child) => body = &child.body,
            TSNamespaceDeclarationBody::TSModuleBlock(block) => return block,
        }
    }
}

fn type_name<'s>(declaration: &'s Declaration<'_>) -> Option<&'s str> {
    match declaration {
        Declaration::TSTypeAliasDeclaration(alias) => Some(alias.id.name.as_str()),
        Declaration::TSInterfaceDeclaration(interface) => Some(interface.id.name.as_str()),
        Declaration::TSEnumDeclaration(enumeration) => Some(enumeration.id.name.as_str()),
        Declaration::ClassDeclaration(class) => class.id.as_ref().map(|id| id.name.as_str()),
        Declaration::TSNamespaceDeclaration(namespace) => Some(namespace.id.name.as_str()),
        Declaration::TSImportEqualsDeclaration(import) => Some(import.id.name.as_str()),
        _ => None,
    }
}

fn declared_name<'s>(statement: &'s Statement<'_>) -> Option<&'s str> {
    match statement {
        Statement::ExportDeclaration(export) => type_name(&export.declaration),
        _ => statement.as_declaration().and_then(type_name),
    }
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn record_namespace_types(
        &mut self,
        statements: &[Statement<'_>],
    ) {
        let mut groups: FxHashMap<_, Vec<_>> = FxHashMap::default();
        let mut exported = FxHashSet::default();
        for statement in statements {
            if let Statement::ExportNamedDeclaration(export) = statement {
                exported.extend(export.specifiers.iter().map(|specifier| {
                    crate::ast::module_export_name(&specifier.local).into_owned()
                }));
            }
            let Some(namespace) = namespace(statement) else {
                continue;
            };
            if matches!(statement, Statement::ExportDeclaration(_)) {
                exported.insert(namespace.id.name.to_string());
            }
            if self
                .literal_objects
                .closed
                .namespace_shadows
                .contains_key(&namespace_block(namespace).span.start)
            {
                continue;
            }
            groups
                .entry(namespace.id.name.as_str())
                .or_default()
                .push(namespace);
        }
        if groups.is_empty() {
            return;
        }
        let module_local = self.scopes.len() == 1
            && statements
                .iter()
                .any(|statement| statement.as_module_declaration().is_some());
        let mut inherited = FxHashSet::default();
        inherited.extend(
            self.literal_objects
                .closed
                .type_frames
                .iter()
                .flat_map(|frame| frame.keys().cloned()),
        );
        inherited.extend(
            statements
                .iter()
                .filter_map(declared_name)
                .map(str::to_string),
        );
        let inherited = Arc::new(inherited);
        let mut pending: Vec<_> = groups
            .into_iter()
            .map(|(name, group)| {
                let names = if module_local && !exported.contains(name) {
                    Names::default()
                } else {
                    Arc::clone(&inherited)
                };
                (group, names)
            })
            .collect();
        while let Some((group, inherited)) = pending.pop() {
            let mut names = (*inherited).clone();
            let mut blocks = Vec::new();
            let mut children: FxHashMap<_, Vec<_>> = FxHashMap::default();
            let mut locals = Vec::new();
            for declaration in group {
                match &declaration.body {
                    TSNamespaceDeclarationBody::TSNamespaceDeclaration(child) => {
                        names.insert(child.id.name.to_string());
                        children
                            .entry(child.id.name.as_str())
                            .or_default()
                            .push(child.as_ref());
                    }
                    TSNamespaceDeclarationBody::TSModuleBlock(block) => {
                        blocks.push(block);
                        let mut local_groups: FxHashMap<_, Vec<_>> = FxHashMap::default();
                        for statement in &block.body {
                            let exported = matches!(statement, Statement::ExportDeclaration(_));
                            if exported || declaration.declare {
                                if let Some(name) = declared_name(statement) {
                                    names.insert(name.to_string());
                                }
                                if let Some(child) = namespace(statement) {
                                    children
                                        .entry(child.id.name.as_str())
                                        .or_default()
                                        .push(child);
                                }
                            } else if let Some(child) = namespace(statement) {
                                local_groups
                                    .entry(child.id.name.as_str())
                                    .or_default()
                                    .push(child);
                            }
                        }
                        locals.extend(local_groups.into_values());
                    }
                }
            }
            let names = Arc::new(names);
            for block in blocks {
                self.literal_objects
                    .closed
                    .namespace_shadows
                    .entry(block.span.start)
                    .or_insert_with(|| Arc::clone(&names));
            }
            pending.extend(
                children
                    .into_values()
                    .chain(locals)
                    .map(|group| (group, Arc::clone(&names))),
            );
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn shadow_namespace_types(
        &mut self,
        block: &TSModuleBlock<'_>,
    ) {
        let names = self
            .literal_objects
            .closed
            .namespace_shadows
            .get(&block.span.start)
            .map(Arc::clone)
            .unwrap_or_else(|| {
                Arc::new(
                    self.literal_objects
                        .closed
                        .type_frames
                        .iter()
                        .flat_map(|frame| frame.keys().cloned())
                        .collect(),
                )
            });
        let own: FxHashSet<_> = block.body.iter().filter_map(declared_name).collect();
        for name in names.iter().filter(|name| !own.contains(name.as_str())) {
            self.shadow_parameter_type(name);
        }
    }
}
