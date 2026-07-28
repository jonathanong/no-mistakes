use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    project_relationship_edges, relationship_projection_fixture, RelationshipProjectionSummary,
};

pub(super) fn bench_relationship_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("relationship_projection");
    for logical_edges in [4_096, 16_384] {
        let fixture = relationship_projection_fixture(logical_edges);
        assert_eq!(
            project_relationship_edges(&fixture),
            RelationshipProjectionSummary {
                projected_edges: logical_edges as usize,
            }
        );
        group.throughput(Throughput::Elements((logical_edges * 2) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(logical_edges),
            &fixture,
            |b, fixture| {
                b.iter(|| black_box(project_relationship_edges(black_box(fixture))));
            },
        );
    }
    group.finish();
}
