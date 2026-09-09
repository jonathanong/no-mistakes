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
