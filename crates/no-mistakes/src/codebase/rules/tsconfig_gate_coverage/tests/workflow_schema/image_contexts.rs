use super::*;

#[test]
fn invalid_runner_and_container_image_fields_do_not_credit_typechecks() {
    let documents = vec![
        workflow(
            ".github/workflows/invalid-runs-on-secret.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: '${{ secrets.RUNNER }}'\n    steps:\n      - run: tsc --noEmit --project invalid-runs-on-secret/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-runs-on-hash-files.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\n    steps:\n      - run: tsc --noEmit --project invalid-runs-on-hash-files/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-container-image.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: 'ghcr.io//checker'}\n    steps:\n      - run: tsc --noEmit --project invalid-container-image/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-static-container-image.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: \"${{ 'node:' }}\"}\n    steps:\n      - run: tsc --noEmit --project invalid-static-container-image/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-reduced-container-image.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: \"node:${{ '' }}\"}\n    steps:\n      - run: tsc --noEmit --project invalid-reduced-container-image/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-digest-algorithm.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: 'node@sha256+1:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'}\n    steps:\n      - run: tsc --noEmit --project invalid-digest-algorithm/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/invalid-docker-action.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: docker://ghcr.io//checker:22\n      - run: tsc --noEmit --project invalid-docker-action/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-dynamic-docker-action.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: 'docker://ghcr.io/checker:${{ matrix.tag }}'\n      - run: tsc --noEmit --project valid-dynamic-docker-action/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid-uppercase-registry.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: 'GHCR.IO/team/checker:22'}\n    steps:\n      - run: tsc --noEmit --project valid-uppercase-registry/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "invalid-runs-on-secret/tsconfig.json",
        "invalid-runs-on-hash-files/tsconfig.json",
        "invalid-container-image/tsconfig.json",
        "invalid-static-container-image/tsconfig.json",
        "invalid-reduced-container-image/tsconfig.json",
        "invalid-digest-algorithm/tsconfig.json",
        "invalid-docker-action/tsconfig.json",
        "valid-dynamic-docker-action/tsconfig.json",
        "valid-uppercase-registry/tsconfig.json",
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
            "valid-dynamic-docker-action/tsconfig.json".to_string(),
            "valid-uppercase-registry/tsconfig.json".to_string(),
        ]),
    );
}
