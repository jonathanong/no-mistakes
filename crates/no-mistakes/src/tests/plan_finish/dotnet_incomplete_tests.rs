use crate::tests::{PlanIncompletenessKind, TestPlan, Warning};

fn warning(kind: &str, file: &str) -> Warning {
    Warning {
        r#type: kind.to_string(),
        message: format!("{kind} for {file}"),
        file: file.to_string(),
        line: None,
    }
}

/// Different files, one missing-baseline condition. A malformed project is a
/// different condition and must stay out of the marker.
#[test]
fn dotnet_missing_baseline_shares_one_kind_when_files_differ() {
    let mut plan = TestPlan {
        warnings: vec![
            warning("package-manifest-no-baseline", "package.json"),
            warning("dotnet-dependency-no-baseline", "App.csproj"),
            warning("dotnet-dependency-no-baseline", "Directory.Packages.props"),
            warning("dotnet-dependency-no-baseline", "packages.lock.json"),
            warning("dotnet-dependency-malformed-xml", "broken.csproj"),
        ],
        ..Default::default()
    };
    plan.finish(false, &[]);

    assert_eq!(plan.incomplete.len(), 1);
    assert_eq!(
        plan.incomplete[0].kind,
        PlanIncompletenessKind::DependencyBaselineMissing
    );
    assert_eq!(
        plan.incomplete[0].files,
        vec![
            "App.csproj".to_string(),
            "Directory.Packages.props".to_string(),
            "package.json".to_string(),
            "packages.lock.json".to_string(),
        ]
    );
    let json = serde_json::to_value(&plan).unwrap();
    assert_eq!(json["incomplete"][0]["kind"], "dependency-baseline-missing");
    assert!(!json["incomplete"][0]["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file == "broken.csproj"));
}

#[test]
fn incompleteness_kind_serializes_as_the_public_name() {
    let kind = PlanIncompletenessKind::DependencyBaselineMissing;
    assert_eq!(kind.as_str(), "dependency-baseline-missing");
    assert_eq!(kind.to_string(), kind.as_str());
    assert_eq!(serde_json::to_value(kind).unwrap(), kind.as_str());
    let restored: PlanIncompletenessKind =
        serde_json::from_str("\"dependency-baseline-missing\"").unwrap();
    assert_eq!(restored, kind);
}
