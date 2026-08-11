use super::*;

fn jobs(yaml: &str) -> serde_yaml::Mapping {
    serde_yaml::from_str::<Value>(yaml)
        .unwrap()
        .get("jobs")
        .and_then(Value::as_mapping)
        .unwrap()
        .clone()
}

#[test]
fn dependency_validation_rejects_malformed_and_unresolvable_jobs() {
    assert!(!valid_job_dependencies(&jobs("jobs:\n  ? [bad]\n  : {}")));
    assert!(!valid_job_dependencies(&jobs("jobs:\n  bad: []")));
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: 1"
    )));
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: [valid, 1]\n  valid: {}"
    )));
    assert!(!valid_job_dependencies(&jobs("jobs:\n  A: {}\n  a: {}")));
    for job_id in ["1build", "build job", "$x"] {
        assert!(!valid_job_dependencies(&jobs(&format!(
            "jobs:\n  '{job_id}': {{}}"
        ))));
    }
    assert!(!valid_job_dependencies(&jobs(
        "jobs:\n  bad:\n    needs: missing"
    )));
}

#[test]
fn step_jobs_reject_unknown_keys() {
    let valid = serde_yaml::from_str::<Value>(
        "runs-on: ubuntu-latest\nsteps:\n  - run: echo valid\ntimeout-minutes: 5",
    )
    .unwrap();
    assert!(jobs::step_job_shape_valid(&valid));

    let unknown = serde_yaml::from_str::<Value>(
        "runs-on: ubuntu-latest\nsteps:\n  - run: echo invalid\nbogus: true",
    )
    .unwrap();
    assert!(!jobs::step_job_shape_valid(&unknown));
}

#[test]
fn job_schema_validates_run_and_reusable_call_field_values() {
    let run_job = serde_yaml::from_str::<Value>(
        "runs-on: [self-hosted, linux]\npermissions:\n  contents: read\nenv: {NODE_ENV: test}\ndefaults:\n  run: {shell: bash}\nconcurrency: {group: checks, cancel-in-progress: false}\noutputs: {result: '${{ steps.run.outputs.result }}'}\nenvironment: {name: staging, url: 'https://example.test'}\ntimeout-minutes: 5\ncontinue-on-error: false\ncontainer:\n  image: node:22\n  credentials: {username: octo, password: '${{ secrets.TOKEN }}'}\n  env: {CI: true}\n  ports: [8080]\n  volumes: ['cache:/data']\n  options: --cpus 1\nservices:\n  postgres:\n    image: postgres:16\n    env: {POSTGRES_PASSWORD: postgres}\nstrategy:\n  fail-fast: false\n  max-parallel: 2\n  matrix: {node: [22]}\nsteps:\n  - id: run\n    run: echo ok",
    )
    .unwrap();
    assert!(scan_job_shape_valid(&run_job));

    let reusable_job = serde_yaml::from_str::<Value>(
        "name: checks\nuses: ./.github/workflows/checks.yml\nwith: {node: 22}\nsecrets: inherit\npermissions: read-all\nconcurrency: checks\nstrategy:\n  matrix: {node: [22]}",
    )
    .unwrap();
    assert!(scan_job_shape_valid(&reusable_job));

    let mixed_expressions = serde_yaml::from_str::<Value>(
        "name: check ${{ github.ref }}\nruns-on: ubuntu-${{ matrix.version }}\nenv: {REF: 'refs/${{ github.ref_name }}'}\noutputs: {result: 'result-${{ steps.run.outputs.value }}'}\nenvironment: {name: 'preview-${{ github.ref_name }}'}\ncontainer: 'node:${{ matrix.node }}'\nservices: {postgres: {image: 'postgres:${{ matrix.postgres }}'}}\nsteps:\n  - name: run ${{ github.ref_name }}\n    working-directory: app/${{ matrix.package }}\n    shell: bash\n    run: tsc --noEmit --project ${{ matrix.project }}",
    )
    .unwrap();
    assert!(scan_job_shape_valid(&mixed_expressions));

    for yaml in [
        "runs-on: ubuntu-latest\nenv: {}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\nsteps:\n  - env: {}\n    run: tsc --noEmit",
    ] {
        let job = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(scan_job_shape_valid(&job), "{yaml}");
    }

    for yaml in [
        "steps:\n  - run: echo invalid",
        "runs-on: true\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\npermissions: bogus\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nenv: [invalid]\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ndefaults: {run: {shell: []}}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nconcurrency: {group: []}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\noutputs: {result: true}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\noutputs: {}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\nenvironment: {url: 'https://example.test'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, ports: [null]}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nservices: {postgres: {image: postgres:16, volumes: [false]}}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nservices: {postgres: postgres:16}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nservices: {}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\nstrategy: {max-parallel: 0}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nstrategy: {}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\nsteps:\n  - run: 'tsc --noEmit ${{ }}'",
        "runs-on: ubuntu-latest\nenv: {REF: '${{ }}'}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\noutputs: {result: '${{ }}'}\nsteps:\n  - run: tsc --noEmit",
        "runs-on: ubuntu-latest\ncontainer: 'node:${{ }}'\nsteps:\n  - run: tsc --noEmit",
        "uses: ./.github/workflows/checks.yml\nwith: {ref: '${{ }}'}",
        "uses: 1",
        "uses: ./.github/workflows/checks.yml\npermissions: bogus",
        "uses: ./.github/workflows/checks.yml\nconcurrency: {group: []}",
    ] {
        let job = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(!scan_job_shape_valid(&job), "{yaml}");
    }
}

#[test]
fn service_names_require_github_identifier_grammar() {
    for name in ["postgres", "postgres_16", "postgres-service"] {
        let job = serde_yaml::from_str::<Value>(&format!(
            "runs-on: ubuntu-latest\nservices:\n  '{name}': {{image: postgres:16}}\nsteps:\n  - run: echo ok"
        ))
        .unwrap();
        assert!(scan_job_shape_valid(&job), "{name}");
    }
    for name in ["1postgres", "postgres service", "postgres!", "pöstgres"] {
        let job = serde_yaml::from_str::<Value>(&format!(
            "runs-on: ubuntu-latest\nservices:\n  '{name}': {{image: postgres:16}}\nsteps:\n  - run: echo invalid"
        ))
        .unwrap();
        assert!(!scan_job_shape_valid(&job), "{name}");
    }
}

#[test]
fn container_credentials_validate_expression_contexts() {
    let valid = serde_yaml::from_str::<Value>(
        "runs-on: ubuntu-latest\ncontainer:\n  image: node:22\n  credentials:\n    username: '${{ github.actor }}'\n    password: '${{ secrets.REGISTRY_TOKEN }}'\nsteps:\n  - run: echo ok",
    )
    .unwrap();
    assert!(scan_job_shape_valid(&valid));

    for credentials in [
        "username: '${{ jobs.build.outputs.username }}'\n    password: token",
        "username: '${{ job.status }}'\n    password: token",
        "username: '${{ runner.os }}'\n    password: token",
        "username: '${{ steps.login.outputs.username }}'\n    password: token",
        "username: octo\n    password: \"${{ hashFiles('**/pnpm-lock.yaml') }}\"",
    ] {
        let job = serde_yaml::from_str::<Value>(&format!(
            "runs-on: ubuntu-latest\ncontainer:\n  image: node:22\n  credentials:\n    {credentials}\nsteps:\n  - run: echo invalid"
        ))
        .unwrap();
        assert!(!scan_job_shape_valid(&job), "{credentials}");
    }
}

#[test]
fn container_fields_validate_their_expression_contexts() {
    for yaml in [
        "runs-on: ubuntu-latest\ncontainer: 'node:${{ steps.setup.outputs.tag }}'\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, env: {TAG: '${{ steps.setup.outputs.tag }}'}}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, ports: ['${{ steps.setup.outputs.port }}']}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, volumes: ['${{ steps.setup.outputs.volume }}']}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\ncontainer: {image: node:22, options: '${{ steps.setup.outputs.options }}'}\nsteps:\n  - run: echo invalid",
    ] {
        let job = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(!scan_job_shape_valid(&job), "{yaml}");
    }

    let valid = serde_yaml::from_str::<Value>(
        "runs-on: ubuntu-latest\ncontainer:\n  image: 'node:${{ matrix.node }}'\n  env: {STATUS: '${{ job.status }}'}\n  ports: ['${{ matrix.port }}']\n  volumes: ['${{ vars.VOLUME }}']\n  options: '${{ inputs.options }}'\nsteps:\n  - run: echo ok",
    )
    .unwrap();
    assert!(scan_job_shape_valid(&valid));

    for volume in [
        "cache:relative",
        "cache",
        ":/data",
        "relative/path:/data",
        "cache:/data:ro",
        "cache:${{ vars.VOLUME }}:ro",
    ] {
        let job = serde_yaml::from_str::<Value>(&format!(
            "runs-on: ubuntu-latest\ncontainer: {{image: node:22, volumes: ['{volume}']}}\nsteps:\n  - run: echo invalid"
        ))
        .unwrap();
        assert!(!scan_job_shape_valid(&job), "{volume}");
    }
    for volume in [
        "/data",
        "cache:/data",
        "/host/data:/data",
        "${{ vars.VOLUME }}",
        "${{ vars.SOURCE }}:/data",
        "cache-${{ vars.SOURCE }}:/data",
        "cache:${{ vars.DESTINATION }}",
    ] {
        let job = serde_yaml::from_str::<Value>(&format!(
            "runs-on: ubuntu-latest\ncontainer: {{image: node:22, volumes: ['{volume}']}}\nsteps:\n  - run: echo valid"
        ))
        .unwrap();
        assert!(scan_job_shape_valid(&job), "{volume}");
    }
}

#[test]
fn scanner_rejects_strategy_fields_with_unavailable_contexts_or_invalid_scalars() {
    for yaml in [
        "runs-on: ubuntu-latest\nstrategy: {fail-fast: '${{ github.ref == ''refs/heads/main'' }}', max-parallel: '${{ needs.setup.outputs.parallel }}'}\nsteps:\n  - run: tsc --noEmit",
        "uses: ./.github/workflows/checks.yml\nstrategy: {fail-fast: '${{ inputs.fail_fast }}', max-parallel: '${{ vars.MAX_PARALLEL }}'}",
    ] {
        let job = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(scan_job_shape_valid(&job), "{yaml}");
    }

    for yaml in [
        "runs-on: ubuntu-latest\nstrategy: {fail-fast: '${{ matrix.enabled }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nstrategy: {max-parallel: '${{ secrets.PARALLEL }}'}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nstrategy: {fail-fast: 'true'}\nsteps:\n  - run: echo invalid",
        "uses: ./.github/workflows/checks.yml\nstrategy: {max-parallel: 1.5}",
        "runs-on: ubuntu-latest\nstrategy: {fail-fast: \"${{ format('{0}', github.ref) }}\"}\nsteps:\n  - run: echo invalid",
        "runs-on: ubuntu-latest\nstrategy: {max-parallel: '${{ contains(github.ref, ''main'') }}'}\nsteps:\n  - run: echo invalid",
        "uses: ./.github/workflows/checks.yml\nstrategy: {max-parallel: '${{ true }}'}",
    ] {
        let job = serde_yaml::from_str::<Value>(yaml).unwrap();
        assert!(!scan_job_shape_valid(&job), "{yaml}");
    }
}

#[test]
fn remote_reusable_targets_require_github_owner_and_repository_names() {
    assert!(canonical_remote_call_target(
        "octo-org/example_repo.name/.github/workflows/checks.yml@v1"
    ));
    for target in [
        "octo_org/repository/.github/workflows/checks.yml@v1",
        "-octo/repository/.github/workflows/checks.yml@v1",
        "octo-/repository/.github/workflows/checks.yml@v1",
        "octo/repo?itory/.github/workflows/checks.yml@v1",
        "octo/../.github/workflows/checks.yml@v1",
    ] {
        assert!(!canonical_remote_call_target(target), "{target}");
    }
}

#[test]
fn remote_references_reject_lock_components() {
    for reference in [
        "foo.lock",
        "refs/heads/foo.lock",
        "release/foo.lock/candidate",
    ] {
        assert!(!valid_remote_reference(reference), "{reference}");
    }
    assert!(valid_remote_reference("release/foo.locked"));
}
