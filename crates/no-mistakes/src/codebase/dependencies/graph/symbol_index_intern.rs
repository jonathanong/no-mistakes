type SourceBucketEntry = (Arc<str>, ImporterRecord);
type SourceBuckets = HashMap<Arc<Path>, Vec<SourceBucketEntry>>;

const MAX_SOURCE_BUCKET_INITIAL_CAPACITY: usize = 8;

fn source_bucket_initial_capacity(expected_entries: usize) -> usize {
    expected_entries.min(MAX_SOURCE_BUCKET_INITIAL_CAPACITY)
}

fn insert_source_bucket_entry(
    source_buckets: &mut SourceBuckets,
    source: Arc<Path>,
    imported_name: Arc<str>,
    importer: ImporterRecord,
    expected_entries: usize,
) {
    source_buckets
        .entry(source)
        .or_insert_with(|| Vec::with_capacity(source_bucket_initial_capacity(expected_entries)))
        .push((imported_name, importer));
}

#[derive(Default)]
struct SymbolIndexInterner {
    paths: HashMap<Arc<Path>, ()>,
    strings: HashMap<Arc<str>, ()>,
}

impl SymbolIndexInterner {
    fn path(&mut self, path: &Path) -> Arc<Path> {
        if let Some((existing, _)) = self.paths.get_key_value(path) {
            return existing.clone();
        }
        let interned = intern_symbol_index_path(path);
        self.paths.insert(interned.clone(), ());
        interned
    }

    fn string(&mut self, value: &str) -> Arc<str> {
        if let Some((existing, _)) = self.strings.get_key_value(value) {
            return existing.clone();
        }
        let interned = intern_symbol_index_str(value);
        self.strings.insert(interned.clone(), ());
        interned
    }
}

pub(crate) fn intern_symbol_index_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(path.as_ref())
}

pub(crate) fn intern_symbol_index_str(value: impl Into<Arc<str>>) -> Arc<str> {
    value.into()
}

fn merge_source_buckets(mut left: SourceBuckets, right: SourceBuckets) -> SourceBuckets {
    for (source, mut entries) in right {
        match left.entry(source) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                let slot = occupied.get_mut();
                slot.reserve(entries.len());
                slot.append(&mut entries);
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                vacant.insert(entries);
            }
        }
    }
    left
}
