use crate::codebase::ts_source::{
    ClassifiedPath, FileClassification, FileId, FileInventory, SourceReadOutcome, SourceStore,
};
use ignore::WalkBuilder;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Snapshot {
    pub root: PathBuf,
    pub sources: SourceStore,
    pub paths: BTreeMap<String, FileId>,
    pub reasons: Vec<String>,
    links: BTreeSet<String>,
    scan_failed: bool,
    side: &'static str,
}

impl Snapshot {
    pub fn discover(root: &Path, side: &'static str) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|error| {
            format!(
                "cannot read {side} root {} ({:?})",
                root.display(),
                error.kind()
            )
        })?;
        if !root.is_dir() {
            return Err(format!(
                "cannot read {side} root {} (not a directory)",
                root.display()
            ));
        }
        let mut entries = Vec::new();
        let mut reasons = Vec::new();
        let mut link_paths = BTreeSet::new();
        for entry in WalkBuilder::new(&root).standard_filters(false).build() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    reasons.push(format!(
                        "cannot enumerate {side} root {}: {error}",
                        root.display()
                    ));
                    continue;
                }
            };
            let file_type = entry.file_type();
            if file_type.is_some_and(|kind| kind.is_dir()) {
                continue;
            }
            if file_type.is_some_and(|kind| kind.is_symlink()) {
                link_paths.insert(entry.path().to_owned());
            }
            let classification = if file_type.is_some_and(|kind| kind.is_file()) {
                FileClassification::TRACKED_REGULAR
            } else {
                FileClassification::default()
            };
            entries.push(ClassifiedPath {
                path: entry.into_path(),
                classification,
            });
        }
        let scan_failed = !reasons.is_empty();
        let inventory = Arc::new(FileInventory::from_classified_paths(entries));
        let mut paths = BTreeMap::new();
        let mut links = BTreeSet::new();
        for path in inventory.paths().iter() {
            let relative = path.strip_prefix(&root).expect("walk stays within root");
            let Some(name) = relative.to_str() else {
                reasons.push(format!("non-UTF-8 {side} path {relative:?}"));
                continue;
            };
            let name = name.replace(std::path::MAIN_SEPARATOR, "/");
            let id = inventory
                .id_for_normalized_path(path)
                .expect("discovered path has a file ID");
            if link_paths.contains(path) {
                links.insert(name.clone());
            }
            paths.insert(name, id);
        }
        Ok(Self {
            root,
            sources: SourceStore::new(inventory),
            paths,
            reasons,
            links,
            scan_failed,
            side,
        })
    }

    pub fn knows(&self, path: &str) -> bool {
        !self.scan_failed || self.paths.contains_key(path)
    }

    pub fn is_link(&self, path: &str) -> bool {
        self.links.contains(path)
    }

    pub fn link_identity(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        if !self.paths.contains_key(path) {
            return Ok(None);
        }
        if !self.is_link(path) {
            return Ok(Some(b"\0file".to_vec()));
        }
        let target = std::fs::read_link(self.root.join(path)).map_err(|error| {
            format!(
                "cannot read {} link {path:?} ({:?})",
                self.side,
                error.kind()
            )
        })?;
        Ok(Some(target.as_os_str().as_encoded_bytes().to_vec()))
    }

    pub fn read(&self, path: &str) -> Result<Option<SourceReadOutcome>, String> {
        let Some(&id) = self.paths.get(path) else {
            return Ok(None);
        };
        let inventory = self.sources.inventory();
        if !inventory
            .classification(id)
            .is_some_and(FileClassification::is_lexical_file)
        {
            return Err(format!(
                "cannot read {} file {path:?} (not a regular file)",
                self.side
            ));
        }
        Ok(Some(
            self.sources
                .read(id)
                .expect("inventory ID has a source slot"),
        ))
    }

    pub fn bytes<'a>(
        &self,
        path: &str,
        source: &'a Option<SourceReadOutcome>,
    ) -> Result<Option<&'a [u8]>, String> {
        source
            .as_ref()
            .map(|result| match result {
                Ok(text) => Ok(text.as_bytes()),
                Err(error) => error
                    .get_ref()
                    .and_then(|inner| inner.downcast_ref::<std::string::FromUtf8Error>())
                    .map(std::string::FromUtf8Error::as_bytes)
                    .ok_or_else(|| format!("cannot read {} file {path:?}: {error}", self.side)),
            })
            .transpose()
    }
}
