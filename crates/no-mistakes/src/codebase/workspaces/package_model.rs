impl WorkspacePackage {
    #[inline(never)]
    fn resolve_subpath(
        &self,
        subpath: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        if let Some(exports) = &self.exports {
            let target = resolve_export_subpath(exports, subpath)?;
            return resolve_workspace_path(&normalize_path(&self.dir.join(target)), visible_files);
        }

        let relative = subpath.strip_prefix("./")?;
        let candidate = normalize_path(&self.dir.join(relative));
        resolve_workspace_path(&candidate, visible_files)
    }

    fn resolve_import(
        &self,
        specifier: &str,
        visible_files: Option<&std::collections::HashSet<PathBuf>>,
    ) -> Option<PathBuf> {
        let imports = self.imports.as_ref()?;
        let target = resolve_export_subpath(imports, specifier)?;
        resolve_workspace_path(&normalize_path(&self.dir.join(target)), visible_files)
    }
}
