use super::*;
use crate::codebase::config::InferredRoots;
use crate::config::v2::schema::{Project, ProjectType};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum RootKind {
    Nextjs,
    Remix,
    Vitejs,
}
type RootCell = Arc<OnceLock<Option<PathBuf>>>;
pub(super) type InferredRootSlots = OnceLock<DashMap<(PathBuf, RootKind), RootCell>>;

impl SourceStore {
    pub(crate) fn inferred_roots_for_projects<'a>(
        &self,
        root: &Path,
        projects: impl Iterator<Item = &'a Project>,
    ) -> InferredRoots {
        let mut inferred = InferredRoots::default();
        for project in projects.filter(|project| project.root.is_none()) {
            let (kind, slot) = match project.type_ {
                Some(ProjectType::Nextjs) => (RootKind::Nextjs, &mut inferred.nextjs),
                Some(ProjectType::Remix) => (RootKind::Remix, &mut inferred.remix),
                Some(ProjectType::Vitejs) => (RootKind::Vitejs, &mut inferred.vitejs),
                _ => continue,
            };
            if slot.is_none() {
                *slot = Some(self.inferred_root(root, kind));
            }
        }
        inferred
    }

    fn inferred_root(&self, root: &Path, kind: RootKind) -> Option<PathBuf> {
        let root = crate::codebase::ts_resolver::normalize_path(root);
        let slot = self
            .inferred_roots
            .get_or_init(DashMap::new)
            .entry((root.clone(), kind))
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone();
        slot.get_or_init(|| {
            let paths = crate::codebase::ts_source::discover_files_from_visible(
                root.as_path(),
                &[],
                &self.inventory.paths(),
            );
            match kind {
                RootKind::Nextjs => {
                    crate::codebase::config::infer_nextjs_root_from_visible(&root, &paths)
                }
                RootKind::Remix => InferredRoots::remix_from_sources(&root, &paths, self),
                RootKind::Vitejs => InferredRoots::vitejs_from_sources(&root, &paths, self),
            }
        })
        .clone()
    }
}

#[cfg(test)]
mod tests;
