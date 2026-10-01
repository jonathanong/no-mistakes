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
        for declared in &facts.declared {
            let used = declared.split('.').any(|segment| {
                facts
                    .value_uses
                    .binary_search_by(|name| name.as_str().cmp(segment))
                    .is_ok()
            });
            if used {
                self.escape(self.path, declared.split('.').next().unwrap_or(declared));
            }
        }
        for name in &facts.value_uses {
            if let Some(binding) = self.index.imported.get(name) {
                self.escape_imported_value(binding);
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

    /// An import read as a value: the namespace behind it, if any, escapes.
    fn escape_imported_value(
        &self,
        binding: &crate::codebase::dependencies::extract::ImportedBinding,
    ) {
        let Some(target) = self.visible_target(self.path, &binding.specifier) else {
            return;
        };
        let export = match binding.kind {
            ImportedBindingKind::Named => binding.imported.as_str(),
            ImportedBindingKind::Default => "default",
            ImportedBindingKind::Namespace => return self.escape_closure(&target),
        };
        match self.resolve_namespace_root(&target, export, &mut Vec::new()) {
            RootLookup::Root(file, root) => self.escape(&file, &root),
            RootLookup::Unresolved => self.escape_closure(&target),
            RootLookup::Other => {}
        }
    }

    /// Every namespace `file` exports or re-exports, directly or through other
    /// modules, escapes: the module is used as a whole.
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
            let exported = index.exported.values().filter_map(|binding| {
                binding.specifier.as_deref().or_else(|| {
                    index
                        .imported
                        .get(&binding.local)
                        .map(|imported| imported.specifier.as_str())
                })
            });
            for specifier in index.stars.iter().map(String::as_str).chain(exported) {
                pending.extend(self.visible_target(&current, specifier));
            }
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
