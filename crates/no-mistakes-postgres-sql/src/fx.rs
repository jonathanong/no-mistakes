pub(crate) use rustc_hash::{FxHashMap, FxHashSet};

#[inline]
pub(crate) fn fx_map<K, V>() -> FxHashMap<K, V> {
    FxHashMap::with_hasher(Default::default())
}
