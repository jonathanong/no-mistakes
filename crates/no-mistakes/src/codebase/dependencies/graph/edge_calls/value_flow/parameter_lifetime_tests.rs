use super::*;

thread_local! {
    static OLD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    // Live compute scopes, overlap at annotation, retired capacity, scopes, annotations.
    static WORK: std::cell::Cell<[usize; 5]> = const { std::cell::Cell::new([0; 5]) };
}

pub(in super::super) fn old_mode() -> bool {
    OLD.with(std::cell::Cell::get)
}

struct ComputationScope;
impl Drop for ComputationScope {
    fn drop(&mut self) {
        WORK.with(|work| {
            let mut counts = work.get();
            assert!(counts[0] > 0);
            counts[0] -= 1;
            work.set(counts);
        });
    }
}

pub(in super::super) fn scope() -> impl Drop {
    WORK.with(|work| {
        let mut counts = work.get();
        counts[0] += 1;
        counts[3] += 1;
        work.set(counts);
    });
    ComputationScope
}

pub(in super::super) fn retiring(capacity: usize) {
    WORK.with(|work| {
        let mut counts = work.get();
        counts[2] += capacity;
        work.set(counts);
    });
}

pub(super) fn annotation() {
    WORK.with(|work| {
        let mut counts = work.get();
        counts[1] = counts[1].max(counts[0]);
        counts[4] += 1;
        work.set(counts);
    });
}

struct Mode(bool);
impl Mode {
    fn new(old: bool) -> Self {
        WORK.with(|work| work.set([0; 5]));
        Self(OLD.with(|mode| mode.replace(old)))
    }
}
impl Drop for Mode {
    fn drop(&mut self) {
        OLD.with(|mode| mode.set(self.0));
    }
}

fn observe(old: bool, fixture: &str, name: &str) -> (Vec<FlowValue>, [usize; 5]) {
    let _mode = Mode::new(old);
    let mut values = Vec::new();
    with_prepared_flow_in(fixture, true, |flow, path, index, parameters| {
        let parameter = parameters.iter().find(|p| p.name == name).unwrap();
        for _ in 0..24 {
            let value = flow.parameter(path, parameter, index);
            assert!(
                has_query(&value)
                    || matches!(&value, FlowValue::Provenance { typed, .. } if !typed.is_empty()),
                "Unknown cannot prove the cold indexed annotation"
            );
            values.push(value);
        }
        assert_eq!(count(), 1);
        flow.cache.clear();
        values.push(flow.parameter(path, parameter, index));
        assert_eq!(count(), 2, "a fresh generation still computes exactly once");
    });
    (values, WORK.with(std::cell::Cell::get))
}

#[test]
fn annotation_retires_nonempty_compute_buffers_before_construction() {
    for (fixture, name) in [
        (
            "test-cases/codebase-analysis/parameter-annotation-cache/fixture",
            "db",
        ),
        (
            "fixtures/benchmarks/parameter-annotation-cache/cold",
            "query",
        ),
    ] {
        let (old_values, old) = observe(true, fixture, name);
        let (new_values, new) = observe(false, fixture, name);
        assert!(old_values == new_values);
        assert_eq!(old[0], 0);
        assert_eq!(new[0], 0);
        assert_eq!(
            old[1], 1,
            "the saved old ordering must observe a live compute scope"
        );
        assert_eq!(
            new[1], 0,
            "annotation must follow computation scope destruction"
        );
        assert!(
            old[2] > 0 && new[2] > 0,
            "empty temporary work cannot prove retirement"
        );
        assert_eq!(old[3..], [2, 2]);
        assert_eq!(new[3..], [2, 2]);
        eprintln!("ACTUAL_PARAMETER_LIFETIME fixture={fixture} old={old:?} new={new:?}");
    }
}
