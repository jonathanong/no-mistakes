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
        // The empty map is allocation-free and lets promotion move both the
        // original key and value without cloning or requiring T: Default.
        let previous = std::mem::replace(self, Self::Many(fx_map()));
        *self = match previous {
            Self::One(existing, _) if existing == name => Self::One(existing, value),
            Self::One(existing, previous) => {
                let mut names = fx_map();
                names.insert(existing, previous);
                names.insert(name, value);
                Self::Many(names)
            }
            Self::Many(mut names) => {
                names.insert(name, value);
                Self::Many(names)
            }
        };
    }

    fn get(&self, name: &str) -> Option<&T> {
        match self {
            Self::One(existing, value) => (existing == name).then_some(value),
            Self::Many(names) => names.get(name),
        }
    }

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
    while let Some(((scope, name), value)) = entries.next() {
        let names = match scopes.entry(scope) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(ScopedNames::One(name, value))
            }
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().insert(name, value);
                entry.into_mut()
            }
        };
        // Extracted bindings are commonly scope-sorted. Reuse the same entry
        // for that run; interleaved scopes still retain normal last-wins lookup.
        while let Some(((_, name), value)) =
            entries.next_if(|((next_scope, _), _)| *next_scope == scope)
        {
            names.insert(name, value);
        }
    }
    scopes
}

fn scoped_name<'a, T>(scopes: &'a ScopedNameMap<T>, scope: usize, name: &str) -> Option<&'a T> {
    scopes.get(&scope)?.get(name)
}
