use super::FileInventory;
use crate::fx::{fx_map, FxHashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Path-addressed storage interned by a frozen [`FileInventory`].
///
/// Hot lookups go through FileId slots. Paths that are not in the inventory
/// (tests and late inserts) land in a small overflow map.
#[derive(Clone)]
pub(crate) struct FileIdMap<T> {
    inventory: Arc<FileInventory>,
    slots: Vec<Option<T>>,
    overflow: FxHashMap<PathBuf, T>,
}

impl<T> Default for FileIdMap<T> {
    fn default() -> Self {
        Self {
            inventory: Arc::new(FileInventory::from_lookup_paths(
                std::iter::empty::<PathBuf>(),
            )),
            slots: Vec::new(),
            overflow: fx_map(),
        }
    }
}

impl<T> FileIdMap<T> {
    pub(crate) fn with_inventory(inventory: Arc<FileInventory>) -> Self {
        let slots = (0..inventory.len()).map(|_| None).collect();
        Self {
            inventory,
            slots,
            overflow: fx_map(),
        }
    }

    pub(crate) fn from_iter_with_inventory(
        entries: impl IntoIterator<Item = (PathBuf, T)>,
        inventory: Arc<FileInventory>,
    ) -> Self {
        let mut map = Self::with_inventory(inventory);
        for (path, value) in entries {
            map.insert(path, value);
        }
        map
    }

    pub(crate) fn from_entries(entries: impl IntoIterator<Item = (PathBuf, T)>) -> Self {
        let entries: Vec<_> = entries.into_iter().collect();
        let inventory = Arc::new(FileInventory::from_lookup_paths(
            entries.iter().map(|(path, _)| path.clone()),
        ));
        Self::from_iter_with_inventory(entries, inventory)
    }

    fn slot_index(&self, path: &Path) -> Option<usize> {
        let id = self.inventory.id_for_path(path)?;
        // Inventory lookup may collapse equivalent spellings. Map keys stay
        // the stored inventory path so alias PathBufs do not appear present.
        (self.inventory.path(id)? == path).then_some(id.index())
    }

    pub(crate) fn get(&self, path: &Path) -> Option<&T> {
        if let Some(index) = self.slot_index(path) {
            if let Some(value) = self.slots.get(index).and_then(Option::as_ref) {
                return Some(value);
            }
        }
        self.overflow.get(path)
    }

    pub(crate) fn get_mut(&mut self, path: &Path) -> Option<&mut T> {
        if let Some(index) = self.slot_index(path) {
            if self.slots.get(index).is_some_and(Option::is_some) {
                return self.slots.get_mut(index).and_then(Option::as_mut);
            }
        }
        self.overflow.get_mut(path)
    }

    pub(crate) fn insert(&mut self, path: PathBuf, value: T) -> Option<T> {
        if let Some(index) = self.slot_index(&path) {
            let previous = self.slots.get_mut(index).and_then(Option::take);
            self.slots[index] = Some(value);
            return previous.or_else(|| self.overflow.remove(&path));
        }
        self.overflow.insert(path, value)
    }

    pub(crate) fn remove(&mut self, path: &Path) -> Option<T> {
        if let Some(index) = self.slot_index(path) {
            let previous = self.slots.get_mut(index).and_then(Option::take);
            return previous.or_else(|| self.overflow.remove(path));
        }
        self.overflow.remove(path)
    }

    pub(crate) fn contains_key(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn len(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count() + self.overflow.len()
    }

    pub(crate) fn keys(&self) -> impl Iterator<Item = &PathBuf> {
        self.inventory
            .as_paths()
            .iter()
            .zip(self.slots.iter())
            .filter_map(|(path, slot)| slot.as_ref().map(|_| path))
            .chain(self.overflow.keys())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&PathBuf, &T)> {
        self.inventory
            .as_paths()
            .iter()
            .zip(self.slots.iter())
            .filter_map(|(path, slot)| slot.as_ref().map(|value| (path, value)))
            .chain(self.overflow.iter())
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &T> {
        self.slots
            .iter()
            .filter_map(Option::as_ref)
            .chain(self.overflow.values())
    }

    pub(crate) fn extend(&mut self, other: Self) {
        for (path, value) in other {
            self.insert(path, value);
        }
    }
}

#[path = "file_id_map_iter.rs"]
mod iter;
