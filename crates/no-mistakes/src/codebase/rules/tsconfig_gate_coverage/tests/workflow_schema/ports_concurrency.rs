use super::*;

#[test]
fn resolved_port_and_concurrency_schema_errors_do_not_credit_typechecks() {
    let documents = vec![
        workflow(
            ".github/workflows/invalid-container-static-port.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix: {port: [70000]}\n    runs-on: ubuntu-latest\n    container: {image: node:22, ports: ['${{ matrix.port }}:6379']}\n    steps:\n      - run: tsc --noEmit --project invalid-container-static-port/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-service-static-port.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix: {port: [70000]}\n    runs-on: ubuntu-latest\n    services: {postgres: {image: postgres:16, ports: ['${{ matrix.port }}:5432']}}\n    steps:\n      - run: tsc --noEmit --project invalid-service-static-port/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-workflow-concurrency-group.yml",
            "on: push\nconcurrency: '${{ '' }}'\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-workflow-concurrency-group/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-job-concurrency-group.yml",
            "on: push\njobs:\n  typecheck:\n    concurrency:\n      group: '${{ '' }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-job-concurrency-group/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/dynamic-port.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: node:22, ports: ['${{ github.ref }}:6379']}\n    steps:\n      - run: tsc --noEmit --project dynamic-port/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "dynamic-port/tsconfig.json",
        "invalid-container-static-port/tsconfig.json",
        "invalid-job-concurrency-group/tsconfig.json",
        "invalid-service-static-port/tsconfig.json",
        "invalid-workflow-concurrency-group/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &tracked,
            &project_inputs(&tracked),
        ),
        BTreeSet::from(["dynamic-port/tsconfig.json".to_string()])
    );
}
