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
    let validate = job_body(&workflow, "validate-rust");
    let rust_cache = validate
        .find("Swatinem/rust-cache")
        .expect("Validate must set up Swatinem/rust-cache before package verification");
    let publish = validate
        .find("cargo publish --dry-run --locked --workspace")
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
fn validate_job_drops_test_executables_after_tests_before_package_verification() {
    // The v0.87.1 release ran every test, then `cargo publish --dry-run` hit
    // `No space left on device`: debug test executables fill the runner disk.
    let workflow = release_workflow();
    let validate = job_body(&workflow, "validate-rust");
    let rust_tests = validate
        .find(
            "bash \"$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh\" test-unit\n",
        )
        .unwrap();
    let integration_tests = validate
        .find("bash \"$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh\" test-integration\n")
        .unwrap();
    let doctests = validate
        .find("bash \"$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh\" doc\n")
        .unwrap();
    // JavaScript tests run independently and do not consume Rust executables.
    let javascript = job_body(&workflow, "validate-js");
    for command in [
        "pnpm run test:ci",
        "pnpm run --if-present test:timeout-policy-browser",
    ] {
        let run = format!("run: {command}\n");
        assert!(javascript.contains(&run));
        assert!(!validate.contains(&run));
    }
    let cleanup_heading = validate
        .find("name: Drop Rust test executables")
        .expect("Validate must free test executables before package verification");
    let publish = validate
        .find("cargo publish --dry-run --locked --workspace")
        .unwrap();
    assert!(
        rust_tests < integration_tests
            && integration_tests < doctests
            && doctests < cleanup_heading
            && cleanup_heading < publish,
        "test executables must be dropped after Rust tests and before cargo publish"
    );
    let step = &validate[cleanup_heading..publish];
    // drop-rust-test-executables.test.cjs pins which files the script keeps.
    assert!(
        step.contains(
            "bash \"$RUNNER_TEMP/validation-tools/.github/scripts/drop-rust-test-executables.sh\" target"
        ),
        "cleanup must use the tested drop script for both Cargo artifact layouts"
    );
    assert!(
        !step.contains("rm -rf target"),
        "cleanup must not wipe whole target directories"
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
    let validate = job_body(&workflow, "validate-rust");
    let reclaim = validate.find("name: Reclaim runner disk space").unwrap();
    let start = validate
        .find("name: Start isolated PostgreSQL for catalog fixtures")
        .expect("Release validation must provision the real catalog-test server");
    let tests = validate.find("name: Run Rust tests").unwrap();
    assert!(
        reclaim < start && start < tests,
        "disk reclamation must finish before provisioning PostgreSQL, and the server must start before Rust tests"
    );
    let setup = &validate[start..tests];
    for required in [
        "timeout-minutes: 5",
        "uses: ./.github/actions/start-test-postgres",
    ] {
        assert!(
            setup.contains(required),
            "release catalog-test setup lost required invariant: {required}"
        );
    }
}

#[test]
fn native_releases_use_thin_lto_and_sixteen_codegen_units_before_cache_restore() {
    let workflow = release_workflow();
    let script = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.github/scripts/cargo-release-build.sh"),
    )
    .unwrap();
    // One cargo invocation emits the rlib and cdylib, then writes timings.
    assert!(script.contains(
        "NO_MISTAKES_BUILD_NAPI=1 cargo build --release --locked --target \"$target\" -p no-mistakes --timings"
    ));
    assert!(script.contains("crate-type = [\"rlib\", \"cdylib\"]"));
    assert!(!script.contains("--crate-type cdylib"));
    let steps = job_body(&workflow, "build-native-linux-x64");
    let (settings, step_body) = steps.split_once("\n    steps:").unwrap();
    assert!(step_body.contains("fromJSON(env.NM_BUILD_TIMEOUT)"));
    for (job, timeout, build_timeout) in [
        ("build-native-linux-x64", "100", "65"),
        ("build-native-linux-arm", "100", "65"),
        ("build-native-darwin", "120", "90"),
        ("build-native-windows", "120", "90"),
    ] {
        let body = job_body(&workflow, job);
        let (job_settings, job_steps) = body.split_once("\n    steps:").unwrap();
        let cache = job_steps.find("Swatinem/rust-cache").unwrap();
        let build = job_steps.find("cargo-release-build.sh").unwrap();
        assert!(
            cache < build,
            "{job} must restore rust-cache before compiling"
        );
        assert!(job_settings.contains(&format!("\n    timeout-minutes: {timeout}\n")));
        for setting in [
            "CARGO_PROFILE_RELEASE_LTO: thin",
            "CARGO_PROFILE_RELEASE_CODEGEN_UNITS: \"16\"",
            &format!("NM_BUILD_TIMEOUT: \"{build_timeout}\""),
        ] {
            assert!(job_settings.contains(&format!("      {setting}")));
        }
        assert!(!job_settings.contains("'fat'"));
        assert!(!job_settings.contains("|| '1'"));
    }
    assert!(settings.contains("CARGO_PROFILE_RELEASE_LTO: thin"));
}
