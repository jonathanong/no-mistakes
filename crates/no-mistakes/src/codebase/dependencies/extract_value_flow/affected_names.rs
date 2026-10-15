use super::*;

/// The physical root and one projected receiver need no name-table buffer.
#[derive(Default)]
pub(in super::super::super) enum Names {
    #[default]
    Empty,
    One(String),
    Two(String, String),
    Many(FxHashSet<String>),
}
impl Names {
    pub(in super::super::super) fn insert(&mut self, name: &str) {
        // A repeated root must not allocate a duplicate owned name.
        if self.contains(name) {
            return;
        }
        match self {
            Self::Empty => *self = Self::One(name.to_owned()),
            Self::One(first) => {
                *self = Self::Two(std::mem::take(first), name.to_owned());
            }
            Self::Two(first, second) => {
                let mut names = fx_set();
                names.insert(std::mem::take(first));
                names.insert(std::mem::take(second));
                names.insert(name.to_owned());
                *self = Self::Many(names);
            }
            Self::Many(names) => {
                names.insert(name.to_owned());
            }
        }
    }
    pub(in super::super::super) fn get(&self, name: &str) -> Option<&String> {
        match self {
            Self::Many(names) => names.get(name),
            _ => self.iter().find(|candidate| candidate.as_str() == name),
        }
    }
    pub(in super::super::super) fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }
    pub(in super::super::super) fn iter(&self) -> impl Iterator<Item = &String> {
        let (first, second, many) = match self {
            Self::Empty => (None, None, None),
            Self::One(first) => (Some(first), None, None),
            Self::Two(first, second) => (Some(first), Some(second), None),
            Self::Many(names) => (None, None, Some(names)),
        };
        first
            .into_iter()
            .chain(second)
            .chain(many.into_iter().flatten())
    }
    #[cfg(feature = "test-instrumentation")]
    pub(super) fn heap_buffers(&self) -> usize {
        match self {
            Self::Many(names) => usize::from(names.capacity() > 0),
            _ => 0,
        }
    }
}
