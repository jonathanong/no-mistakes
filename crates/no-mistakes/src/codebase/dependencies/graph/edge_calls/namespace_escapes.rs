/// `(file, root)` pairs whose namespace is used in a way the graph cannot
/// follow to a class. `"*"` stands for every root of the file.
type NamespaceEscapes = FxHashMap<std::path::PathBuf, FxHashSet<String>>;

impl CallSiteResolution<'_, '_> {
    /// Records every use of a namespace in `file` that the graph does not
    /// resolve to a class: an alias, an argument, a computed access, `export
    /// default`, a merged declaration, a module read whole or at runtime.
    /// Uses it does resolve, and misses found while resolving, are recorded
    /// as they happen.
    fn record_namespace_escapes(&self, file: &TsFileFacts) {
        let facts = &file.namespaces;
        for root in facts.roots.iter().filter(|root| root.merged) {
            self.escape(self.path, &root.name);
        }
        // A value use names a declared path (`Errors.Inner`) or an import
        // local; a same-named value that is neither never reaches here.
        for used in &facts.value_uses {
            if facts.declared.binary_search(used).is_ok() {
                self.escape(self.path, used.split('.').next().unwrap_or(used));
            }
            if let Some(binding) = self.index.imported.get(used) {
                self.escape_imported_value(binding, None);
            }
        }
        for (local, member) in &facts.member_uses {
            if let Some(binding) = self.index.imported.get(local) {
                self.escape_imported_value(binding, Some(member));
            }
        }
        let runtime_imports = file
            .imports
            .iter()
            .filter(|import| matches!(import.kind, ImportKind::Dynamic | ImportKind::Require))
            .map(|import| import.specifier.as_str());
        for specifier in runtime_imports.chain(facts.opaque_specifiers.iter().map(String::as_str)) {
            if let Some(target) = self.visible_target(self.path, specifier) {
                self.escape_closure(&target);
            }
        }
    }

    /// An import read as a value: the namespace behind it, if any, escapes. A
    /// namespace import read through one static member (`mod.version`) reads
    /// only that export, so only a namespace it names escapes; the module
    /// object itself, handed on whole, exposes everything it exports.
    fn escape_imported_value(
        &self,
        binding: &crate::codebase::dependencies::extract::ImportedBinding,
        member: Option<&str>,
    ) {
        let Some(target) = self.visible_target(self.path, &binding.specifier) else {
            return;
        };
        let export = match (binding.kind, member) {
            (ImportedBindingKind::Named, _) => binding.imported.as_str(),
            (ImportedBindingKind::Default, _) => "default",
            (ImportedBindingKind::Namespace, Some(member)) => member,
            (ImportedBindingKind::Namespace, None) => return self.escape_closure(&target),
        };
        match self.resolve_namespace_root(&target, export, &mut Vec::new()) {
            RootLookup::Root(file, root) => self.escape(&file, &root),
            RootLookup::Unresolved => self.escape_closure(&target),
            RootLookup::Other => {}
        }
    }

    /// Every namespace `file` exports, directly or through re-exports, escapes:
    /// the module is used as a whole. Each named export is followed to the
    /// namespace it exposes, so a module that re-exports only a value does not
    /// reach the namespaces its source declares. `export *` and an export that
    /// cannot be followed to a namespace take their whole module.
    fn escape_closure(&self, file: &std::path::Path) {
        let mut seen: FxHashSet<std::path::PathBuf> = FxHashSet::default();
        let mut pending = vec![file.to_path_buf()];
        while let Some(current) = pending.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            self.escape(&current, "*");
            let Some(index) = self.indexes.file(self.facts, &current) else {
                continue;
            };
            for (name, binding) in &index.exported {
                match self.resolve_namespace_root(&current, name, &mut Vec::new()) {
                    RootLookup::Root(root_file, root) => self.escape(&root_file, &root),
                    RootLookup::Other => {}
                    RootLookup::Unresolved => {
                        let imported = index.imported.get(&binding.local);
                        let specifier = binding
                            .specifier
                            .as_deref()
                            .or(imported.map(|imported| imported.specifier.as_str()));
                        pending.extend(specifier.and_then(|s| self.visible_target(&current, s)));
                    }
                }
            }
            let stars = index.stars.iter();
            pending.extend(stars.filter_map(|s| self.visible_target(&current, s)));
        }
    }
}

/// Marks each class whose namespace escaped, in `escapes` or in its own file.
fn mark_escaped_namespaces(classes: &mut [ClassDeclaration], escapes: &NamespaceEscapes) {
    for class in classes {
        let root = class
            .namespace
            .as_deref()
            .map(|path| path.split('.').next().unwrap_or(path));
        class.namespace_escaped = escapes.get(&class.file).is_some_and(|roots| {
            root.is_some_and(|root| roots.contains(root) || roots.contains("*"))
        });
    }
}
