use crate::codebase::ts_source::SourceStore;
use crate::codebase::workspaces::WorkspaceMap;
use std::path::Path;

pub(super) struct WorkspaceManifestScope<'a> {
    root: &'a Path,
    workspace_map: &'a WorkspaceMap,
    globs: Vec<(bool, globset::GlobMatcher)>,
}

impl<'a> WorkspaceManifestScope<'a> {
    pub(super) fn prepare(
        root: &'a Path,
        workspace_map: &'a WorkspaceMap,
        sources: &SourceStore,
    ) -> Self {
        let globs =
            crate::codebase::workspaces::load_workspace_globs_from_source_store(root, sources)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|glob| {
                    let (excluded, pattern) = glob
                        .strip_prefix('!')
                        .map_or((false, glob.as_str()), |pattern| (true, pattern));
                    let pattern = crate::codebase::glob_normalize::normalize(pattern);
                    globset::GlobBuilder::new(&pattern)
                        .literal_separator(true)
                        .build()
                        .ok()
                        .map(|glob| (excluded, glob.compile_matcher()))
                })
                .collect();
        Self {
            root,
            workspace_map,
            globs,
        }
    }

    pub(super) fn contains(&self, manifest: &Path) -> bool {
        manifest == self.root.join("package.json")
            || self
                .workspace_map
                .packages
                .iter()
                .any(|package| manifest == package.dir.join("package.json"))
            || self.matches_glob(manifest)
    }

    fn matches_glob(&self, manifest: &Path) -> bool {
        let Some(directory) = manifest.parent() else {
            return false;
        };
        let Ok(relative) = directory.strip_prefix(self.root) else {
            return false;
        };
        let mut included = false;
        for (excluded, glob) in &self.globs {
            if glob.is_match(relative) {
                if *excluded {
                    return false;
                }
                included = true;
            }
        }
        included
    }
}
