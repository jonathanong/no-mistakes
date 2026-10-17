// Measurement-only append to extract_collector.rs AFTER production binaries freeze.
// No field/representation changes; never committed to analyzer source.
#[cfg(receiver_layout_probe)]
pub fn receiver_layout_probe() -> [(&'static str, usize, usize); 6] {
    use crate::codebase::dependencies::graph::ResolvedCallSite;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use std::mem::{align_of, size_of};
    [
        (
            "ImportCollector",
            size_of::<ImportCollector>(),
            align_of::<ImportCollector>(),
        ),
        (
            "ValueFlowFacts",
            size_of::<ValueFlowFacts>(),
            align_of::<ValueFlowFacts>(),
        ),
        (
            "ImportFacts",
            size_of::<ImportFacts>(),
            align_of::<ImportFacts>(),
        ),
        (
            "TsFileFacts",
            size_of::<TsFileFacts>(),
            align_of::<TsFileFacts>(),
        ),
        ("ValueCall", size_of::<ValueCall>(), align_of::<ValueCall>()),
        (
            "ResolvedCallSite",
            size_of::<ResolvedCallSite>(),
            align_of::<ResolvedCallSite>(),
        ),
    ]
}
