use super::FileIdMap;
use std::path::PathBuf;

impl<T> FileIdMap<T> {
    pub(crate) fn into_entries(self) -> impl Iterator<Item = (PathBuf, T)> {
        let Self {
            inventory,
            slots,
            overflow,
        } = self;
        let mut entries: Vec<_> = slots
            .into_iter()
            .zip(inventory.as_paths())
            .filter_map(|(slot, path)| slot.map(|value| (path.clone(), value)))
            .collect();
        entries.extend(overflow);
        entries.into_iter()
    }
}

impl<T> IntoIterator for FileIdMap<T> {
    type Item = (PathBuf, T);
    type IntoIter = std::vec::IntoIter<(PathBuf, T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_entries().collect::<Vec<_>>().into_iter()
    }
}

impl<'a, T> IntoIterator for &'a FileIdMap<T> {
    type Item = (&'a PathBuf, &'a T);
    type IntoIter = std::vec::IntoIter<(&'a PathBuf, &'a T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter().collect::<Vec<_>>().into_iter()
    }
}

impl<T> std::iter::FromIterator<(PathBuf, T)> for FileIdMap<T> {
    fn from_iter<I: IntoIterator<Item = (PathBuf, T)>>(iter: I) -> Self {
        Self::from_entries(iter)
    }
}

impl<T> From<std::collections::HashMap<PathBuf, T>> for FileIdMap<T> {
    fn from(map: std::collections::HashMap<PathBuf, T>) -> Self {
        Self::from_entries(map)
    }
}

impl<T, const N: usize> From<[(PathBuf, T); N]> for FileIdMap<T> {
    fn from(entries: [(PathBuf, T); N]) -> Self {
        Self::from_entries(entries)
    }
}

impl<T> std::ops::Index<&PathBuf> for FileIdMap<T> {
    type Output = T;

    fn index(&self, path: &PathBuf) -> &Self::Output {
        self.get(path).expect("fact path is not present")
    }
}

impl<'a, T> IntoIterator for &'a mut FileIdMap<T> {
    type Item = (&'a PathBuf, &'a mut T);
    type IntoIter = FileIdMapIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        FileIdMapIterMut {
            paths: self.inventory.as_paths(),
            slots: self.slots.iter_mut(),
            overflow: self.overflow.iter_mut(),
            index: 0,
        }
    }
}

pub(crate) struct FileIdMapIterMut<'a, T> {
    paths: &'a [PathBuf],
    slots: std::slice::IterMut<'a, Option<T>>,
    overflow: std::collections::hash_map::IterMut<'a, PathBuf, T>,
    index: usize,
}

impl<'a, T> Iterator for FileIdMapIterMut<'a, T> {
    type Item = (&'a PathBuf, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        for slot in self.slots.by_ref() {
            let path = &self.paths[self.index];
            self.index += 1;
            if let Some(value) = slot.as_mut() {
                return Some((path, value));
            }
        }
        self.overflow.next()
    }
}
