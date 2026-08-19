use criterion::{black_box, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    analyze_react_traits_file, react_traits_many_components_fixture, ReactTraitsSummary,
};

pub(super) const EXPECTED_COMPONENTS: usize = 32;
pub(super) const EXPECTED_WITH_STATE: usize = 4;
pub(super) const EXPECTED_WITH_PROPS: usize = 8;
pub(super) const EXPECTED_WITH_MEMO: usize = 4;
pub(super) const EXPECTED_WITH_CONTEXT: usize = 4;
pub(super) const EXPECTED_WITH_SUSPENSE: usize = 4;
pub(super) const EXPECTED_WITH_FETCH: usize = 4;
pub(super) const EXPECTED_WITH_CHILDREN: usize = 4;

pub(super) fn bench_react_traits(c: &mut Criterion) {
    let fixture = react_traits_many_components_fixture();
    let preflight = analyze_react_traits_file(&fixture);
    assert_eq!(
        preflight,
        ReactTraitsSummary {
            components: EXPECTED_COMPONENTS,
            with_state: EXPECTED_WITH_STATE,
            with_props: EXPECTED_WITH_PROPS,
            with_memo: EXPECTED_WITH_MEMO,
            with_context: EXPECTED_WITH_CONTEXT,
            with_suspense: EXPECTED_WITH_SUSPENSE,
            with_fetch: EXPECTED_WITH_FETCH,
            with_children: EXPECTED_WITH_CHILDREN,
        },
        "react-traits many-components preflight drifted: {preflight:?}"
    );

    let mut group = c.benchmark_group("react_traits");
    group.throughput(Throughput::Elements(EXPECTED_COMPONENTS as u64));
    group.bench_function("many_components", |b| {
        b.iter(|| black_box(analyze_react_traits_file(black_box(&fixture))));
    });
    group.finish();
}
