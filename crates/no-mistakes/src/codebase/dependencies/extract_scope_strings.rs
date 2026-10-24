use crate::fx::{fx_set, FxHashSet};
use std::sync::Arc;

use super::CallableId;

type SharedScopeCatalogs = (Vec<Arc<str>>, Vec<Arc<str>>, Vec<(CallableId, Arc<str>)>);

/// The three finalized catalogs overlap heavily, but have distinct membership
/// and order. Share only their strings, after collection and sorting finish.
pub(super) fn intern_scope_names(
    known: Vec<String>,
    callable: Vec<String>,
    identities: Vec<(CallableId, String)>,
) -> SharedScopeCatalogs {
    let mut strings: FxHashSet<Arc<str>> = fx_set();
    let mut intern = |value: String| {
        if let Some(shared) = strings.get(value.as_str()) {
            return Arc::clone(shared);
        }
        let shared: Arc<str> = value.into();
        strings.insert(Arc::clone(&shared));
        shared
    };
    // Allocate compact output buffers explicitly: an in-place String -> Arc<str>
    // collect can retain the wider String backing allocation.
    let mut shared_known = Vec::with_capacity(known.len());
    for scope in known {
        shared_known.push(intern(scope));
    }
    let mut shared_callable = Vec::with_capacity(callable.len());
    for scope in callable {
        shared_callable.push(intern(scope));
    }
    let mut shared_identities = Vec::with_capacity(identities.len());
    for (id, scope) in identities {
        shared_identities.push((id, intern(scope)));
    }
    (shared_known, shared_callable, shared_identities)
}
