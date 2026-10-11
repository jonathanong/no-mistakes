use criterion::{black_box, Criterion};
use no_mistakes::codebase::workflow_topology::{
    artifact_types::ArtifactValue, artifact_values::artifact_value, value_primitives,
};

pub(super) fn bench_artifact_values(c: &mut Criterion) {
    let fixture: serde_yaml::Value = serde_yaml::from_str(include_str!(
        "../../../../fixtures/performance/workflow-artifact-matrix.yml"
    ))
    .expect("valid workflow artifact matrix benchmark fixture");
    let matrix = value_primitives::to_json(
        fixture
            .get("matrix")
            .expect("matrix fixture must contain its matrix object"),
    );

    assert_eq!(
        artifact_value("release-artifact", None),
        ArtifactValue::Static {
            raw: "release-artifact".to_string(),
            value: "release-artifact".to_string(),
            instance_count: None,
        }
    );
    assert_eq!(
        artifact_value("release-${{ github.ref_name }}", Some(&matrix)),
        ArtifactValue::Dynamic {
            raw: "release-${{ github.ref_name }}".to_string(),
        }
    );
    let ArtifactValue::Finite {
        values,
        instance_counts,
        ..
    } = artifact_value(
        "release-${{ matrix.os }}-node-${{ matrix.node }}",
        Some(&matrix),
    )
    else {
        panic!("matrix benchmark input must have finite values");
    };
    let expected = vec![
        "release-macos-latest-node-20",
        "release-macos-latest-node-22",
        "release-ubuntu-latest-node-20",
        "release-ubuntu-latest-node-22",
        "release-windows-latest-node-20",
        "release-windows-latest-node-22",
    ];
    assert_eq!(values, expected);
    assert_eq!(instance_counts.len(), 6);
    assert!(instance_counts.values().all(|count| *count == 1));

    let mut group = c.benchmark_group("workflow_artifact_value");
    group.bench_function("static", |b| {
        b.iter(|| black_box(artifact_value(black_box("release-artifact"), None)));
    });
    group.bench_function("dynamic_expression", |b| {
        b.iter(|| {
            black_box(artifact_value(
                black_box("release-${{ github.ref_name }}"),
                Some(black_box(&matrix)),
            ))
        });
    });
    group.bench_function("matrix_expansion", |b| {
        b.iter(|| {
            black_box(artifact_value(
                black_box("release-${{ matrix.os }}-node-${{ matrix.node }}"),
                Some(black_box(&matrix)),
            ))
        });
    });
    group.finish();
}
