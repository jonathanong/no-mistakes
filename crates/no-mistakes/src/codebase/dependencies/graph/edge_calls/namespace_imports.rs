use crate::codebase::dependencies::extract::{ExportedBinding, ImportedBindingKind};

/// What an export name of a module is, for finding the namespace behind it.
#[derive(Clone)]
enum RootLookup {
    /// The namespace root `name` declared in `file`.
    Root(std::path::PathBuf, String),
    /// Not a namespace of this repository.
    Other,
    /// Might be a namespace, but the chain cannot be followed to one: a cycle,
    /// an ambiguous `export *`, or `export * as ns`.
    Unresolved,
}

impl CallSiteResolution<'_, '_> {
    fn visible_target(
        &self,
        from: &std::path::Path,
        specifier: &str,
    ) -> Option<std::path::PathBuf> {
        let resolved = self.resolver.resolve(specifier, from)?;
        let visible = self.edge_inputs.graph_files.visible_path(&resolved)?;
        Some(visible.to_path_buf())
    }

    /// `new Lib.Deep.X()` through an import: `Lib` is followed through the
    /// target's re-exports to a namespace, and `Deep.X` is looked up in it. A
    /// namespace that lacks the member counts as escaped, and so does one the
    /// chain cannot reach, because the construction might build any class.
    fn imported_namespace_member(&self, callee: &str) -> Option<ResolvedCallTarget> {
        let (head, rest) = callee.split_once('.')?;
        let binding = self.index.imported.get(head)?;
        let (export, member) = match binding.kind {
            ImportedBindingKind::Named => (binding.imported.as_str(), rest),
            ImportedBindingKind::Default => ("default", rest),
            ImportedBindingKind::Namespace => rest.split_once('.')?,
        };
        let target = self.visible_target(self.path, &binding.specifier)?;
        match self.resolve_namespace_root(&target, export, &mut Vec::new()) {
            RootLookup::Root(file, root) => {
                let table = &self.indexes.file(self.facts, &file)?.namespaces;
                let Some((scope, id)) = table.members.get(&format!("{root}.{member}")) else {
                    self.escape(&file, &root);
                    return None;
                };
                module_export_target(self.index, callee, Some((file, scope.clone())), Some(*id))
            }
            RootLookup::Unresolved => {
                self.escape_closure(&target);
                None
            }
            RootLookup::Other => None,
        }
    }

    /// The namespace root that `file` exports as `export`, following named and
    /// `export *` re-exports. The visited list is path-scoped: meeting a name
    /// already on the path is a cycle, which cannot be resolved.
    fn resolve_namespace_root(
        &self,
        file: &std::path::Path,
        export: &str,
        visited: &mut Vec<(std::path::PathBuf, String)>,
    ) -> RootLookup {
        let key = (file.to_path_buf(), export.to_string());
        let cacheable = visited.is_empty();
        if cacheable {
            if let Some(hit) = self.indexes.namespace_roots.get(&key) {
                return hit.clone();
            }
        }
        if visited.contains(&key) {
            return RootLookup::Unresolved;
        }
        visited.push(key.clone());
        let result = self.namespace_root_in(file, export, visited);
        visited.pop();
        if cacheable {
            self.indexes.namespace_roots.insert(key, result.clone());
        }
        result
    }

    fn namespace_root_in(
        &self,
        file: &std::path::Path,
        export: &str,
        visited: &mut Vec<(std::path::PathBuf, String)>,
    ) -> RootLookup {
        let Some(index) = self.indexes.file(self.facts, file) else {
            return RootLookup::Unresolved;
        };
        if let Some(root) = index.namespaces.root_exported_as(export) {
            return RootLookup::Root(file.to_path_buf(), root.to_string());
        }
        if let Some(binding) = index.exported.get(export) {
            return self.through_export(file, &index, binding, visited);
        }
        // `export *` never carries `default`.
        if export == "default" {
            return RootLookup::Other;
        }
        let mut roots = Vec::new();
        for specifier in &index.stars {
            match self.follow(file, specifier, export, visited) {
                RootLookup::Root(root_file, root) => roots.push((root_file, root)),
                RootLookup::Other => {}
                RootLookup::Unresolved => return RootLookup::Unresolved,
            }
        }
        roots.sort();
        roots.dedup();
        match (roots.pop(), roots.is_empty()) {
            (Some((root_file, root)), true) => RootLookup::Root(root_file, root),
            (Some(_), false) => RootLookup::Unresolved,
            (None, _) => RootLookup::Other,
        }
    }

    /// An export that is not a namespace declared in `file` itself: a named
    /// re-export, or a re-exported import.
    fn through_export(
        &self,
        file: &std::path::Path,
        index: &CallableFileIndex,
        binding: &ExportedBinding,
        visited: &mut Vec<(std::path::PathBuf, String)>,
    ) -> RootLookup {
        if let Some(specifier) = &binding.specifier {
            // `export * as ns from` exports the whole module as one value.
            return match binding.local.as_str() {
                "*" => RootLookup::Unresolved,
                local => self.follow(file, specifier, local, visited),
            };
        }
        let Some(imported) = index.imported.get(&binding.local) else {
            return RootLookup::Other;
        };
        match imported.kind {
            ImportedBindingKind::Named => {
                self.follow(file, &imported.specifier, &imported.imported, visited)
            }
            ImportedBindingKind::Default => {
                self.follow(file, &imported.specifier, "default", visited)
            }
            ImportedBindingKind::Namespace => RootLookup::Unresolved,
        }
    }

    /// Looks `export` up in the module `specifier` names. A module outside the
    /// repository holds none of its namespaces; a relative one the graph does
    /// not see is unknown.
    fn follow(
        &self,
        from: &std::path::Path,
        specifier: &str,
        export: &str,
        visited: &mut Vec<(std::path::PathBuf, String)>,
    ) -> RootLookup {
        match self.visible_target(from, specifier) {
            Some(target) => self.resolve_namespace_root(&target, export, visited),
            None if external_module_specifier(specifier) => RootLookup::Other,
            None => RootLookup::Unresolved,
        }
    }
}
