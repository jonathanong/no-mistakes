impl PreparedTestProjects {
    #[doc(hidden)]
    pub fn tsconfig_candidate_roots(&self, root: &Path) -> Vec<PathBuf> {
        let mut roots = self
            .projects
            .values()
            .filter_map(|projects| projects.as_ref().ok())
            .flat_map(|projects| projects.iter())
            .flat_map(|project| {
                let scope = project.scope.as_deref().map(|scope| root.join(scope));
                let config_dir = project
                    .config
                    .as_deref()
                    .map(|config| root.join(config))
                    .and_then(|config| config.parent().map(Path::to_path_buf));
                scope.into_iter().chain(config_dir)
            })
            .collect::<Vec<_>>();
        roots.sort();
        roots.dedup();
        roots
    }
}
