use std::path::Path;

impl super::ScopedImportResolver<'_> {
    pub(crate) fn matches_paths_alias(&self, specifier: &str, importing_file: &Path) -> bool {
        if let Some(resolver) = self.fixed_resolver_for(importing_file) {
            return resolver.matches_alias(specifier);
        }
        self.resolver_for(importing_file).matches_alias(specifier)
    }
}
