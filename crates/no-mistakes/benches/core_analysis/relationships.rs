use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};
use no_mistakes::benchmark_support::{
    project_all_relationship_edges, project_relationship_edges, relationship_construction_fixture,
    relationship_index_from_fixture, relationship_projection_fixture,
    RelationshipProjectionSummary,
};

pub(super) fn bench_relationship_projection(c: &mut Criterion) {
    let mut group = c.benchmark_group("relationship_projection");
    for logical_edges in [4_096, 16_384] {
        let construction_fixture = relationship_construction_fixture(logical_edges);
        let fixture = relationship_projection_fixture(logical_edges);
        assert_eq!(
            project_relationship_edges(&fixture),
            RelationshipProjectionSummary {
                projected_edges: logical_edges as usize,
            }
        );
        assert_eq!(
            project_all_relationship_edges(&fixture),
            RelationshipProjectionSummary {
                projected_edges: logical_edges as usize,
            }
        );
        group.throughput(Throughput::Elements((logical_edges * 2) as u64));
        group.bench_with_input(
            BenchmarkId::new("index_construction", logical_edges),
            &construction_fixture,
            |b, fixture| {
                b.iter_batched(
                    || fixture.clone(),
                    |fixture| black_box(relationship_index_from_fixture(fixture)),
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("scoped_projection", logical_edges),
            &fixture,
            |b, fixture| {
                b.iter(|| black_box(project_relationship_edges(black_box(fixture))));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("unscoped_projection", logical_edges),
            &fixture,
            |b, fixture| {
                b.iter(|| black_box(project_all_relationship_edges(black_box(fixture))));
            },
        );
    }
    group.finish();
}
