use super::*;

#[test]
fn native_frontend_fixture_exercises_production_collectors() {
    let fixture = native_frontend_fixture();
    assert_eq!(
        collect_swift_frontend_facts(&fixture),
        NativeFrontendSummary {
            files: 15,
            parsed_files: 5,
            physical_reads: 7,
        }
    );
    assert_eq!(
        collect_dotnet_frontend_facts(&fixture),
        NativeFrontendSummary {
            files: 12,
            parsed_files: 5,
            physical_reads: 7,
        }
    );
}
