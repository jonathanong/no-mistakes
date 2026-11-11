use super::super::cause::Cause;
use crate::codebase::dependencies::graph::NodeId;
use crate::codebase::ts_source::module_facts::arguments::PathDefault;

pub(in super::super) enum Step {
    Block(Cause),
    Seed,
    Follow(
        NodeId,
        usize,
        Option<String>,
        Cause,
        u32,
        Option<PathDefault>,
        bool,
        bool,
    ),
}
