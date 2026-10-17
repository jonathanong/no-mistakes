/// Empty root captures retain their scope boundary, but cannot publish a phase
/// catalog: no target point can append an event and no first change is present.
pub(super) fn capture(segments: &[(u32, u32, u32)], has_roots: bool) -> Vec<(u32, u32, u32)> {
    #[cfg(all(test, feature = "test-instrumentation"))]
    let clone = has_roots || tests::OLD_CLONE.with(std::cell::Cell::get);
    #[cfg(not(all(test, feature = "test-instrumentation")))]
    let clone = has_roots;
    #[cfg(all(test, feature = "test-instrumentation"))]
    tests::COUNTS.with(|n| {
        let mut counts = n.get();
        counts[0] += 1;
        counts[1] += usize::from(has_roots);
        counts[2] += usize::from(clone);
        counts[3] += if clone { segments.len() } else { 0 };
        n.set(counts);
    });
    if clone {
        segments.to_vec()
    } else {
        Vec::new()
    }
}

#[cfg(all(test, feature = "test-instrumentation"))]
#[path = "root_phase_segments_tests.rs"]
mod tests;
