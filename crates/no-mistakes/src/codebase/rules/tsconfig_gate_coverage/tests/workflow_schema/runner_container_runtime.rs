use super::*;

#[test]
fn resolved_runner_strategy_and_container_configuration_control_coverage() {
    let documents = vec![
        workflow(
            ".github/workflows/caller.yml",
            "on: push\njobs:\n  invalid-parallel:\n    uses: ./.github/workflows/parallel-invalid.yml\n    with: {parallelism: 0}\n  valid-parallel:\n    uses: ./.github/workflows/parallel-valid.yml\n    with: {parallelism: 1}\n  omitted-secret:\n    uses: ./.github/workflows/credentials-missing.yml\n  omitted-service-secret:\n    uses: ./.github/workflows/service-credentials-missing.yml\n  available-secret:\n    uses: ./.github/workflows/credentials-available.yml\n    secrets: inherit\n",
        ),
        workflow(
            ".github/workflows/parallel-invalid.yml",
            "on:\n  workflow_call:\n    inputs:\n      parallelism: {type: number, required: true}\njobs:\n  typecheck:\n    strategy:\n      max-parallel: '${{ inputs.parallelism }}'\n      matrix: {target: [one]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project invalid-parallel/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/parallel-valid.yml",
            "on:\n  workflow_call:\n    inputs:\n      parallelism: {type: number, required: true}\njobs:\n  typecheck:\n    strategy:\n      max-parallel: '${{ inputs.parallelism }}'\n      matrix: {target: [one]}\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project valid-parallel/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/credentials-missing.yml",
            "on:\n  workflow_call:\n    secrets:\n      token: {required: false}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container:\n      image: ghcr.io/example/checker:latest\n      credentials:\n        username: checker\n        password: '${{ secrets.token }}'\n    steps:\n      - run: tsc --noEmit --project missing-credential/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/credentials-available.yml",
            "on:\n  workflow_call:\n    secrets:\n      token: {required: false}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container:\n      image: ghcr.io/example/checker:latest\n      credentials:\n        username: checker\n        password: '${{ secrets.token }}'\n    steps:\n      - run: tsc --noEmit --project available-credential/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/service-credentials-missing.yml",
            "on:\n  workflow_call:\n    secrets:\n      token: {required: false}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    services:\n      registry:\n        image: ghcr.io/example/checker:latest\n        credentials:\n          username: checker\n          password: '${{ secrets.token }}'\n    steps:\n      - run: tsc --noEmit --project missing-service-credential/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/empty-credential.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container:\n      image: ghcr.io/example/checker:latest\n      credentials: {username: checker, password: \"${{ '' }}\"}\n    steps:\n      - run: tsc --noEmit --project empty-credential/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unsupported-container-option.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: node:22, options: '--entrypoint /bin/false'}\n    steps:\n      - run: tsc --noEmit --project unsupported-container-option/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/unsupported-service-option.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    services:\n      postgres: {image: postgres:16, options: '--network host'}\n    steps:\n      - run: tsc --noEmit --project unsupported-service-option/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/supported-option.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: node:22, options: '--cpus 1'}\n    steps:\n      - run: tsc --noEmit --project supported-option/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/matrix-runner.yml",
            "on: push\njobs:\n  linux:\n    strategy:\n      matrix: {os: [ubuntu-latest]}\n    runs-on: '${{ matrix.os }}'\n    steps:\n      - run: tsc --noEmit --project matrix-linux/tsconfig.json\n  windows:\n    strategy:\n      matrix: {os: [windows-latest]}\n    runs-on: '${{ matrix.os }}'\n    steps:\n      - run: tsc --noEmit --project matrix-windows/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/runner-groups.yml",
            "on: push\njobs:\n  group-only:\n    runs-on: {group: ubuntu-runners}\n    defaults: {run: {shell: bash}}\n    steps:\n      - run: tsc --noEmit --project runner-group/tsconfig.json\n  group-implicit-shell:\n    runs-on: {group: ubuntu-latest}\n    steps:\n      - run: tsc --noEmit --project runner-group-implicit/tsconfig.json\n  group-labels:\n    runs-on: {group: ubuntu-runners, labels: ubuntu-latest}\n    steps:\n      - run: tsc --noEmit --project runner-group-labels/tsconfig.json\n  labels-only:\n    runs-on: {labels: [ubuntu-latest]}\n    steps:\n      - run: tsc --noEmit --project runner-labels/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "available-credential/tsconfig.json",
        "empty-credential/tsconfig.json",
        "invalid-parallel/tsconfig.json",
        "matrix-linux/tsconfig.json",
        "matrix-windows/tsconfig.json",
        "missing-credential/tsconfig.json",
        "missing-service-credential/tsconfig.json",
        "runner-group-labels/tsconfig.json",
        "runner-group-implicit/tsconfig.json",
        "runner-group/tsconfig.json",
        "runner-labels/tsconfig.json",
        "supported-option/tsconfig.json",
        "unsupported-container-option/tsconfig.json",
        "unsupported-service-option/tsconfig.json",
        "valid-parallel/tsconfig.json",
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
        BTreeSet::from([
            "available-credential/tsconfig.json".to_string(),
            "matrix-linux/tsconfig.json".to_string(),
            "runner-group/tsconfig.json".to_string(),
            "runner-labels/tsconfig.json".to_string(),
            "supported-option/tsconfig.json".to_string(),
            "valid-parallel/tsconfig.json".to_string(),
        ])
    );
}

#[test]
fn resolved_container_and_service_volumes_control_coverage() {
    let documents = vec![workflow(
        ".github/workflows/container-volumes.yml",
        "on: push\njobs:\n  invalid-container:\n    strategy:\n      matrix: {volume: ['./cache']}\n    runs-on: ubuntu-latest\n    container: {image: node:22, volumes: ['${{ matrix.volume }}:/cache']}\n    steps:\n      - run: tsc --noEmit --project invalid-container-volume/tsconfig.json\n  invalid-service:\n    strategy:\n      matrix: {volume: ['./postgres']}\n    runs-on: ubuntu-latest\n    services:\n      postgres: {image: postgres:16, volumes: ['${{ matrix.volume }}:/data']}\n    steps:\n      - run: tsc --noEmit --project invalid-service-volume/tsconfig.json\n  valid-named:\n    strategy:\n      matrix: {volume: [cache]}\n    runs-on: ubuntu-latest\n    container: {image: node:22, volumes: ['${{ matrix.volume }}:/cache']}\n    steps:\n      - run: tsc --noEmit --project valid-named-volume/tsconfig.json\n  valid-absolute:\n    strategy:\n      matrix: {volume: [/host/cache]}\n    runs-on: ubuntu-latest\n    services:\n      postgres: {image: postgres:16, volumes: ['${{ matrix.volume }}:/data']}\n    steps:\n      - run: tsc --noEmit --project valid-absolute-volume/tsconfig.json\n",
    )];
    let projects = [
        "invalid-container-volume/tsconfig.json",
        "invalid-service-volume/tsconfig.json",
        "valid-absolute-volume/tsconfig.json",
        "valid-named-volume/tsconfig.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        ci_typechecked_projects(
            &ParsedWorkflowSet { documents },
            &projects,
            &project_inputs(&projects),
        ),
        BTreeSet::from([
            "valid-absolute-volume/tsconfig.json".to_string(),
            "valid-named-volume/tsconfig.json".to_string(),
        ])
    );
}
