//! Location-owned reference edges and checkpoint pins for bounded old-object reclamation.
mod callbacks;
mod diagnostics;
mod graph;
mod hooks;
mod pins;
mod trial;
mod values;
use super::{transactions::Cell, Environment};
use crate::fx::{FxHashMap, FxHashSet};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(super) enum Location {
    Cell(Cell),
    Mapping(Environment, u64),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(super) enum Owner {
    Frame(Environment),
    Argument(u64),
}
#[derive(Default, Eq, PartialEq)]
pub(super) struct References {
    arguments: FxHashSet<u64>,
    builders: FxHashSet<u64>,
}
#[derive(Default)]
pub(super) struct Reclamation {
    observer: std::sync::OnceLock<Option<std::sync::Arc<crate::diagnostics::InvocationObserver>>>,
    peaks: [usize; 5],
    pending_reference_work: [u64; 3],
    locations: FxHashMap<Location, References>,
    owners: FxHashMap<Owner, FxHashSet<Location>>,
    incoming: FxHashMap<u64, usize>,
    roots: FxHashMap<u64, usize>,
    pins: std::collections::BTreeMap<u64, usize>,
    holders: FxHashMap<u64, FxHashSet<Cell>>,
    masks: FxHashMap<u64, FxHashSet<Cell>>,
    callback_keys: FxHashMap<Environment, FxHashSet<Cell>>,
    candidates: FxHashSet<u64>,
    builder_candidates: FxHashSet<u64>,
}
impl Location {
    fn owner(&self) -> Owner {
        match self {
            Self::Cell(Cell::Binding(env, _)) | Self::Mapping(env, _) => Owner::Frame(*env),
            Self::Cell(Cell::Dense(id, _) | Cell::Extra(id, _)) => Owner::Argument(*id),
            _ => unreachable!("reference-bearing primary location"),
        }
    }
}
