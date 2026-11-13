use serde_yaml::Value;

fn ci_workflow() -> Value {
    serde_yaml::from_str(include_str!("../../../.github/workflows/ci.yml")).unwrap()
}

fn step_names(steps: &[Value]) -> Vec<&str> {
    steps
        .iter()
        .map(|step| step["name"].as_str().expect("CI step must have a name"))
        .collect()
}

fn step_named<'a>(steps: &'a [Value], name: &str) -> &'a Value {
    steps
        .iter()
        .find(|step| step["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("missing CI step: {name}"))
}

fn assert_catalog_postgres_setup(setup: &str) {
    for required in [
        "postgresql-18",
        "extension/citext.control",
        "extension/pg_stat_statements.control",
        "initdb",
        "pg_ctl",
        "listen_addresses=",
        "NO_MISTAKES_TEST_POSTGRES_URL=",
        "host=$RUNNER_TEMP&port=56225",
        "PGOPTIONS=-cstandard_conforming_strings=off",
        "$GITHUB_ENV",
        "$GITHUB_PATH",
    ] {
        assert!(
            setup.contains(required),
            "catalog-test setup lost required invariant: {required}"
        );
    }
}

#[test]
fn standalone_lib_job_starts_catalog_postgres_before_uninstrumented_lib_tests() {
    let workflow = ci_workflow();
    let job = &workflow["jobs"]["standalone-shards"];
    assert_eq!(job["runs-on"].as_str(), Some("ubuntu-latest"));
    let steps = job["steps"].as_sequence().unwrap();
    let names = step_names(steps);
    let postgres = names
        .iter()
        .position(|name| *name == "Start isolated PostgreSQL for catalog fixtures")
        .expect("standalone --lib tests panic in CI without the real PostgreSQL lane");
    let tests = names
        .iter()
        .position(|name| *name == "Test standalone Rust library without a CLI build")
        .expect("standalone job must keep the uninstrumented --lib check");
    assert!(
        postgres < tests,
        "catalog PostgreSQL must start before standalone library tests"
    );

    assert_eq!(
        steps[postgres]["uses"].as_str(),
        Some("./.github/actions/start-test-postgres")
    );
    let action = include_str!("../../../.github/actions/start-test-postgres/action.yml");
    let installer = include_str!("../../../.github/actions/start-test-postgres/setup.py");
    assert_catalog_postgres_setup(&format!("{action}\n{installer}"));
    assert!(action.contains("setup.py"));
    assert_eq!(steps[postgres]["timeout-minutes"].as_u64(), Some(5));

    let test_step = &steps[tests];
    let run = test_step["run"].as_str().unwrap();
    assert_eq!(
        run,
        "node .github/scripts/standalone-shards.cjs run target/standalone-bundle ${{ matrix.shard }} 4"
    );
    assert_eq!(job["needs"].as_str(), Some("standalone-build"));
    let producer = &workflow["jobs"]["standalone-build"];
    let build = step_named(
        producer["steps"].as_sequence().unwrap(),
        "Build standalone library without a CLI",
    );
    assert!(build["run"].as_str().unwrap().lines().any(|line| {
        line.trim() == "node .github/scripts/standalone-shards.cjs build target/standalone-bundle"
    }));
    let helper = include_str!("../../../.github/scripts/standalone-shards.cjs");
    assert!(helper.contains("delete childEnv[\"CARGO_BIN_EXE_no-mistakes\"]"));
    assert!(helper.contains("standalone library validation must not build the CLI"));
    assert_eq!(job["env"]["CARGO_INCREMENTAL"].as_str(), Some("0"));
    assert_eq!(
        job["env"]["CARGO_TARGET_DIR"].as_str(),
        Some("${{ github.workspace }}/target/standalone-lib/target")
    );
}

#[test]
fn tests_gate_requires_standalone_lib_and_coverage_keeps_catalog_postgres() {
    let workflow = ci_workflow();
    let gate = &workflow["jobs"]["tests-gate"];
    assert_eq!(gate["runs-on"].as_str(), Some("ubuntu-slim"));
    let needs = gate["needs"].as_sequence().unwrap();
    assert!(
        needs
            .iter()
            .any(|name| name.as_str() == Some("tests-standalone-lib")),
        "Tests with coverage must still require the standalone --lib job"
    );

    let coverage_steps = workflow["jobs"]["rust-coverage-shards"]["steps"]
        .as_sequence()
        .unwrap();
    let coverage_postgres = step_named(
        coverage_steps,
        "Start isolated PostgreSQL for catalog fixtures",
    );
    assert_eq!(
        coverage_postgres["uses"].as_str(),
        Some("./.github/actions/start-test-postgres")
    );
    assert_eq!(coverage_postgres["timeout-minutes"].as_u64(), Some(5));
    let quality_steps = workflow["jobs"]["quality-source"]["steps"]
        .as_sequence()
        .unwrap();
    let policy = step_named(quality_steps, "Test PostgreSQL setup policy");
    assert_eq!(policy["timeout-minutes"].as_u64(), Some(2));
    assert_eq!(
        policy["run"].as_str(),
        Some("python3 -m unittest discover -s .github/actions/start-test-postgres -p 'test_*.py'")
    );
}
