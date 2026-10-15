use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) struct Package {
    pub name: String,
    pub targets: Vec<Target>,
    entries: BTreeMap<PathBuf, Vec<usize>>,
    scopes: BTreeMap<PathBuf, Vec<usize>>,
}
impl Package {
    pub(super) fn new(name: String, targets: Vec<Target>, directory: &Path) -> Self {
        let mut entries: BTreeMap<PathBuf, Vec<usize>> = BTreeMap::new();
        let mut scopes: BTreeMap<PathBuf, Vec<usize>> = BTreeMap::new();
        for (id, target) in targets.iter().enumerate() {
            entries.entry(target.entry.clone()).or_default().push(id);
            scopes
                .entry(target.entry.with_extension(""))
                .or_default()
                .push(id);
            let parent = target.entry.parent().expect("target entries have a parent");
            // Flat integration/bin entrypoints do not own unrelated sibling helpers.
            if !(target.flag == "--test" && parent == directory.join("tests"))
                && !(target.flag == "--bin" && parent == directory.join("src/bin"))
            {
                scopes.entry(parent.to_path_buf()).or_default().push(id);
            }
        }
        Self {
            name,
            targets,
            entries,
            scopes,
        }
    }
    pub(super) fn is_test_entry(&self, file: &Path) -> bool {
        self.entries
            .get(file)
            .is_some_and(|ids| ids.iter().any(|id| self.targets[*id].flag == "--test"))
    }
    pub(super) fn target_ids(&self, file: &Path) -> &[usize] {
        if let Some(ids) = self.entries.get(file) {
            return ids;
        }
        file.ancestors()
            .skip(1)
            .find_map(|parent| self.scopes.get(parent))
            .filter(|ids| ids.len() == 1)
            .map_or(&[], Vec::as_slice)
    }
}
pub(super) struct Target {
    pub name: String,
    pub entry: PathBuf,
    pub flag: &'static str,
    pub enabled: bool,
}
