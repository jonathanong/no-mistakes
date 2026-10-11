type ScopedNameMap<T> = FxHashMap<usize, ScopedNames<T>>;

// Most lexical frames contain a single callable binding. Keep that binding
// inline rather than allocating a hash table for every frame.
#[derive(Clone)]
enum ScopedNames<T> {
    One(String, T),
    Many(FxHashMap<String, T>),
}

impl<T> ScopedNames<T> {
    fn insert(&mut self, name: String, value: T) {
        match self {
            Self::Many(names) => {
                names.insert(name, value);
            }
            Self::One(existing, previous) if existing == &name => *previous = value,
            Self::One(_, _) => self.extend(std::iter::once((name, value))),
        }
    }

    fn extend(&mut self, rows: impl ExactSizeIterator<Item = (String, T)>) {
        if let Self::Many(names) = self {
            names.reserve(rows.len());
            names.extend(rows);
            return;
        }
        // The empty map is allocation-free and lets promotion move both the
        // original key and value without cloning or requiring T: Default.
        let previous = std::mem::replace(self, Self::Many(fx_map()));
        if let Self::One(name, value) = previous {
            let mut names = crate::fx::fx_map_with_capacity(rows.len() + 1);
            names.insert(name, value);
            names.extend(rows);
            *self = Self::Many(names);
        }
    }

    fn get(&self, name: &str) -> Option<&T> {
        match self {
            Self::One(existing, value) => (existing == name).then_some(value),
            Self::Many(names) => names.get(name),
        }
    }

    #[cfg(any(test, feature = "test-instrumentation"))]
    fn len(&self) -> usize {
        match self {
            Self::One(_, _) => 1,
            Self::Many(names) => names.len(),
        }
    }

    fn values(&self) -> impl Iterator<Item = &T> {
        let (one, many) = match self {
            Self::One(_, value) => (Some(value), None),
            Self::Many(names) => (None, Some(names.values())),
        };
        one.into_iter().chain(many.into_iter().flatten())
    }
}

fn index_scoped_names<T>(
    entries: impl IntoIterator<Item = ((usize, String), T)>,
) -> ScopedNameMap<T> {
    let mut scopes: ScopedNameMap<T> = fx_map();
    let mut entries = entries.into_iter().peekable();
    while let Some(((scope, name), mut value)) = entries.next() {
        // Duplicate-only runs still fit inline and need no temporary vector.
        while let Some(((_, _), next_value)) = entries
            .next_if(|((next_scope, next_name), _)| *next_scope == scope && next_name == &name)
        {
            value = next_value;
        }
        if let Some(((_, next_name), next_value)) =
            entries.next_if(|((next_scope, _), _)| *next_scope == scope)
        {
            // Extracted bindings are commonly scope-sorted. Collect one owned
            // run so the table reserves once, without pre-counting or cloning.
            let mut rows = vec![(name, value), (next_name, next_value)];
            while let Some(((_, name), value)) =
                entries.next_if(|((next_scope, _), _)| *next_scope == scope)
            {
                rows.push((name, value));
            }
            match scopes.entry(scope) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(ScopedNames::Many(rows.into_iter().collect()));
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(rows.into_iter());
                }
            }
        } else {
            // Revisited scopes keep insertion order and last-wins semantics.
            match scopes.entry(scope) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(ScopedNames::One(name, value));
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().insert(name, value);
                }
            }
        }
    }
    scopes
}

fn scoped_name<'a, T>(scopes: &'a ScopedNameMap<T>, scope: usize, name: &str) -> Option<&'a T> {
    scopes.get(&scope)?.get(name)
}
