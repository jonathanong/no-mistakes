//! Fast maps for interned analysis keys.
//!
//! `FxHashMap`/`FxHashSet` skip SipHash. Keys are interned local paths and
//! node ids, not untrusted network input. Public adjacency order is still
//! imposed by explicit sorts after insert.
//!
//! rustc-hash 2 aliases these to `std` maps with `FxBuildHasher`, so `new()`
//! and `with_capacity()` are not available. Use [`fx_map`], [`fx_set`], and
//! [`fx_map_with_capacity`] instead.

pub(crate) use rustc_hash::{FxHashMap, FxHashSet};

/// Interned local-path membership set. Skip SipHash; keys are not untrusted.
pub(crate) type PathSet = FxHashSet<std::path::PathBuf>;

#[inline]
pub(crate) fn fx_map<K, V>() -> FxHashMap<K, V> {
    FxHashMap::with_hasher(Default::default())
}

#[inline]
pub(crate) fn fx_set<K>() -> FxHashSet<K> {
    FxHashSet::with_hasher(Default::default())
}

#[inline]
pub(crate) fn fx_map_with_capacity<K, V>(capacity: usize) -> FxHashMap<K, V> {
    FxHashMap::with_capacity_and_hasher(capacity, Default::default())
}
