use std::path::PathBuf;

fn release_workflow() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

fn job_body<'a>(workflow: &'a str, name: &str) -> &'a str {
    let heading = format!("\n  {name}:\n");
    let start = workflow
        .find(&heading)
        .unwrap_or_else(|| panic!("missing `{name}` job"));
    let body_start = start + heading.len();
    let body = &workflow[body_start..];
    let end = body
        .match_indices('\n')
        .find_map(|(offset, _)| {
            let line = body[offset + 1..]
                .split_once('\n')
                .map(|(line, _)| line)
                .unwrap_or(&body[offset + 1..]);
            is_two_space_job_heading(line).then_some(offset)
        })
        .unwrap_or(body.len());
    &body[..end]
}

fn is_two_space_job_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("  ") else {
        return false;
    };
    if rest.starts_with(' ') {
        return false;
    }
    rest.strip_suffix(':').is_some_and(|name| {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })
}

#[test]
fn validate_job_drops_cargo_package_tree_before_rust_cache_save() {
    let workflow = release_workflow();
    let validate = job_body(&workflow, "validate");
    let rust_cache = validate
        .find("Swatinem/rust-cache")
        .expect("Validate must set up Swatinem/rust-cache before package verification");
    let publish = validate
        .find("cargo publish --dry-run --locked -p no-mistakes")
        .expect("Validate must dry-run cargo publish");
    let cleanup_heading = validate
        .find("name: Drop cargo package verification tree")
        .expect("Validate must drop the cargo package tree before rust-cache save");
    let cleanup = validate[cleanup_heading..]
        .find("rm -rf target/package")
        .map(|offset| cleanup_heading + offset)
        .expect(
            "cleanup must remove target/package so rust-cache does not opendir missing tests/target and tests/trybuild paths",
        );
    assert!(
        rust_cache < publish && publish < cleanup_heading && cleanup_heading < cleanup,
        "package verification tree must be removed after cargo publish --dry-run"
    );
    let step = &validate[cleanup_heading..cleanup];
    assert!(
        step.contains("if: always()"),
        "cleanup must run even when cargo publish fails after creating the tree"
    );
}

#[test]
fn job_body_stops_at_the_next_two_space_job_heading() {
    let workflow = concat!(
        "jobs:\n",
        "  validate:\n",
        "    steps:\n",
        "      - uses: Swatinem/rust-cache@v2\n",
        "      - run: cargo publish --dry-run --locked -p no-mistakes\n",
        "  build-cli:\n",
        "    steps:\n",
        "      - run: echo other job\n",
    );
    let validate = job_body(workflow, "validate");
    assert!(validate.contains("Swatinem/rust-cache"));
    assert!(!validate.contains("echo other job"));
    assert!(job_body(workflow, "build-cli").contains("echo other job"));
}

#[test]
fn validate_job_starts_isolated_catalog_postgres_after_disk_reclamation_before_tests() {
    let workflow = release_workflow();
    let validate = job_body(&workflow, "validate");
    let reclaim = validate.find("name: Reclaim runner disk space").unwrap();
    let start = validate
        .find("name: Start isolated PostgreSQL for catalog fixtures")
        .expect("Release validation must provision the real catalog-test server");
    let tests = validate.find("name: Run Rust tests").unwrap();
    assert!(reclaim < start && start < tests, "disk reclamation must finish before provisioning PostgreSQL, and the server must start before Rust tests");
    let setup = &validate[start..tests];
    for required in [
        "timeout-minutes: 5",
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
            "release catalog-test setup lost required invariant: {required}"
        );
    }
}

#[test]
fn macos_release_uses_parallel_codegen_and_thin_lto_before_cache_restore() {
    let workflow = release_workflow();
    let native = job_body(&workflow, "build-native");
    // A cold macOS release exceeded 45 minutes with fat LTO and one codegen unit.
    // Keep the overrides job-wide so cache identity and compilation agree.
    let (job_settings, steps) = native.split_once("\n    steps:\n").unwrap();
    let (_, job_env) = job_settings.split_once("\n    env:\n").unwrap();
    let env_lines: Vec<_> = job_env
        .lines()
        .take_while(|line| !line.starts_with("    ") || line.starts_with("      "))
        .collect();
    for setting in [
        "CARGO_PROFILE_RELEASE_LTO: ${{ matrix.target == 'aarch64-apple-darwin' && 'thin' || 'fat' }}",
        "CARGO_PROFILE_RELEASE_CODEGEN_UNITS: ${{ matrix.target == 'aarch64-apple-darwin' && '16' || '1' }}",
    ] {
        assert!(
            env_lines.contains(&format!("      {setting}").as_str()),
            "missing job-level release profile override: {setting}"
        );
    }
    assert!(steps.contains("Swatinem/rust-cache"));
    assert!(steps.contains("NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked"));
    assert!(native.contains("timeout-minutes: 45"));
}
