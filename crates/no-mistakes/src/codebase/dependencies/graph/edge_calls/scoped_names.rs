type ScopedNameMap<T> = FxHashMap<usize, FxHashMap<String, T>>;

fn index_scoped_names<T>(
    entries: impl IntoIterator<Item = ((usize, String), T)>,
) -> ScopedNameMap<T> {
    let mut scopes: ScopedNameMap<T> = fx_map();
    for ((scope, name), value) in entries {
        scopes
            .entry(scope)
            .or_insert_with(fx_map)
            .insert(name, value);
    }
    scopes
}

fn scoped_name<'a, T>(scopes: &'a ScopedNameMap<T>, scope: usize, name: &str) -> Option<&'a T> {
    scopes.get(&scope)?.get(name)
}
