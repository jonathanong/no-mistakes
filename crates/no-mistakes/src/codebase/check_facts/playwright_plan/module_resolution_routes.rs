impl PlaywrightModuleResolution {
    pub(crate) fn resolved_path(&self, imported: &str, importing_file: &Path) -> Option<std::path::PathBuf> {
        match self.identity(imported, importing_file) {
            Some(ModuleIdentity::Path(path)) => Some(path),
            _ => None,
        }
    }

    pub(crate) fn strict_modules_match(&self, root: &Path, configured: &str, imported: &str, importing_file: &Path) -> bool {
        let configured_path = crate::codebase::ts_resolver::normalize_path(&root.join(configured));
        if self.visible_files.contains(&configured_path) {
            return self.resolved_path(imported, importing_file).as_ref() == Some(&configured_path);
        }
        match (self.identity(configured, importing_file), self.identity(imported, importing_file)) {
            (Some(configured), Some(imported)) => configured == imported,
            _ => false,
        }
    }

    pub(crate) fn catalog(&self, root: &Path) -> Arc<crate::codebase::ts_resolver::TsConfigCatalog> {
        match &self.tsconfig {
            PlaywrightTsConfig::Single(config) => Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(root, config.as_ref().clone(), None)),
            PlaywrightTsConfig::Catalog => Arc::clone(&self.catalog_resolver.as_ref().expect("catalog facade").catalog),
        }
    }
}
