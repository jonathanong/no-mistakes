//! Write-proportional evaluator checkpoints. No checkpoint clones a state table.
mod access;
mod finish;
mod freshness;
mod indexes;
mod mutations;
mod promotion;
mod prune;
mod read;
mod restore;
mod roots;
use super::frame_arena::FrameCheckpoint;
use super::{Environment, Value};
use crate::fx::FxHashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(super) enum Cell {
    Binding(Environment, String),
    Dense(u64, usize),
    Length(u64),
    Extra(u64, usize),
    Invalidated(u64),
    Builder(u64),
    Deleted(u64, Option<usize>),
    Definite(u64, usize),
    Disconnected(u64, usize),
    Recreated(u64, usize),
    Fresh(Environment, String),
    Trust,
    Module(std::path::PathBuf),
    CallbackSeen(super::callbacks::CallbackIdentity),
    CallbackRunning(super::callbacks::CallbackIdentity),
    CallbackEnabled,
    ExecutionsEnabled,
}
#[derive(Clone, Eq, PartialEq)]
pub(super) enum Saved {
    Value(Option<Value>),
    Flag(bool),
    Length(usize),
    Environment(Option<Environment>),
}
#[derive(Clone, Copy)]
pub(super) struct Checkpoint(pub usize);
pub(super) struct Frame {
    pub scopes: FrameCheckpoint,
    pub identities: u64,
    pub old: FxHashMap<Cell, Saved>,
    pub reference_pins: crate::fx::FxHashSet<u64>,
    pub writes: crate::fx::FxHashSet<Cell>,
    pub root: bool,
    pub recorded: u64,
    pub values_cloned: u64,
    pub rebased_values_cloned: u64,
    pub promoted_frames: crate::fx::FxHashSet<Environment>,
    pub promoted_ids: crate::fx::FxHashSet<u64>,
}
#[derive(Default)]
pub(in crate::codebase::postgres::query_annotation) struct Transactions {
    pub(super) frames: Vec<Frame>,
    pub(super) index_oracle: index_oracle::State,
    pub(super) reclamation: super::reclamation::Reclamation,
}
pub(super) struct Change {
    pub cell: Cell,
    pub before: Saved,
    pub after: Saved,
}

pub(super) struct Delta {
    pub changes: Vec<Change>,
    pub pins: crate::fx::FxHashSet<u64>,
}

#[cfg(test)]
#[path = "transactions/tests/index_oracle.rs"]
mod index_oracle;

#[cfg(not(test))]
#[path = "transactions/index_oracle_runtime.rs"]
mod index_oracle;
