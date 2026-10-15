use super::*;
use std::{
    mem::{align_of, size_of},
    path::PathBuf,
    sync::Arc,
};

// Compare with the actual parent field contracts on this target/compiler, in
// both ordinary native tests and the instrumented coverage build. The added
// execution-quality bits must fit existing padding rather than grow each fact.
struct ParentCall {
    _offset: u32,
    _end: u32,
    _source_callee: String,
    _context: Option<CallableId>,
    _callee: ValueExpression,
    _arguments: Arc<Vec<ValueExpression>>,
    _bound: bool,
    _spread_at: Option<usize>,
    _once: bool,
}
struct ParentReceiverRead {
    _file: PathBuf,
    _offset: u32,
    _context: Option<CallableId>,
    _invocation: bool,
    _owned_context: Option<Box<(PathBuf, CallableId, bool)>>,
}
struct ParentInvocationRead {
    _file: PathBuf,
    _offset: u32,
    _context: Option<CallableId>,
    _owner: (PathBuf, CallableId),
    _once: bool,
}

#[test]
fn repeated_execution_metadata_preserves_fact_and_frame_layouts() {
    // Transient request state is separate from retained call facts and frames.
    eprintln!(
        "execution layouts: ValueCall={} ReceiverRead={} InvocationRead={} transient-parent={} transient-qualified={}",
        size_of::<ValueCall>(),
        size_of::<default_origins::ReceiverRead>(),
        size_of::<default_origins::InvocationRead>(),
        size_of::<Option<(CallableId, u32, Option<CallableId>)>>(),
        size_of::<
            Option<(
                &std::path::Path,
                CallableId,
                execution_quality::ExecutionPhase
            )>,
        >(),
    );
    assert_eq!(
        (size_of::<ValueCall>(), align_of::<ValueCall>()),
        (size_of::<ParentCall>(), align_of::<ParentCall>())
    );
    assert_eq!(
        (
            size_of::<default_origins::ReceiverRead>(),
            align_of::<default_origins::ReceiverRead>()
        ),
        (
            size_of::<ParentReceiverRead>(),
            align_of::<ParentReceiverRead>()
        )
    );
    assert_eq!(
        (
            size_of::<default_origins::InvocationRead>(),
            align_of::<default_origins::InvocationRead>()
        ),
        (
            size_of::<ParentInvocationRead>(),
            align_of::<ParentInvocationRead>()
        )
    );
}
