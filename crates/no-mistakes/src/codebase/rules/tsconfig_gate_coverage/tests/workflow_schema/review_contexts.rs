use super::*;

#[test]
fn invalid_field_contexts_mounts_and_dispatch_ids_earn_no_credit() {
    let documents = vec![
        workflow(
            ".github/workflows/job-name.yml",
            "on: push\njobs:\n  typecheck:\n    name: '${{ jobs.build.result }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project job-name/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/step-name.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - name: '${{ jobs.build.result }}'\n        run: tsc --noEmit --project step-name/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/volume.yml",
            "on: push\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    container: {image: node:22, volumes: ['cache:relative']}\n    steps:\n      - run: tsc --noEmit --project volume/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/concurrency.yml",
            "on: push\nconcurrency: {group: checks, cancel-in-progress: \"${{ 'false' }}\"}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project concurrency/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/matrix.yml",
            "on: push\njobs:\n  typecheck:\n    strategy:\n      matrix: '${{ jobs.build.result }}'\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project matrix/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/dispatch.yml",
            "on:\n  push:\n  workflow_dispatch:\n    inputs:\n      'bad name': {type: string}\njobs:\n  typecheck:\n    runs-on: ubuntu-latest\n    steps:\n      - run: tsc --noEmit --project dispatch/tsconfig.json\n",
        ),
        workflow(
            ".github/workflows/valid.yml",
            "on: push\nconcurrency: {group: checks, cancel-in-progress: true}\njobs:\n  typecheck:\n    name: 'check ${{ github.ref_name }}'\n    runs-on: ubuntu-latest\n    container: {image: node:22, volumes: ['/data']}\n    steps:\n      - name: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"\n        run: tsc --noEmit --project valid/tsconfig.json\n",
        ),
    ];
    let tracked = [
        "job-name/tsconfig.json",
        "step-name/tsconfig.json",
        "volume/tsconfig.json",
        "concurrency/tsconfig.json",
        "matrix/tsconfig.json",
        "dispatch/tsconfig.json",
        "valid/tsconfig.json",
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
        BTreeSet::from(["valid/tsconfig.json".to_string()])
    );
}
