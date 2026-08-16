#[test]
fn collect_remaining_edges_parallelizes_independent_kinds() {
    let remaining = include_str!("../builder_remaining_edges.rs");
    let independent = include_str!("../builder_remaining_edges_independent.rs");
    assert!(
        remaining.contains("collect_independent_remaining_edges"),
        "remaining-edge orchestration must collect the independent language panel"
    );
    assert!(
        independent.contains("rayon::join"),
        "markdown vs terraform/dotnet/swift must collect via rayon::join"
    );
    assert!(
        independent.contains("collect_md_edges"),
        "independent panel must collect markdown edges"
    );
    assert!(
        independent.contains("collect_terraform_edges_for_plan")
            && independent.contains("collect_dotnet_edges_for_plan")
            && independent.contains("collect_swift_edges_for_plan"),
        "independent panel must collect terraform/dotnet/swift beside markdown"
    );
}
