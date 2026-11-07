use std::path::PathBuf;
use std::process::{Command, Output};

fn run(root: &std::path::Path, jobs: &str, mode: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["check", "--format", "json", "--jobs", jobs, "--root"])
        .arg(root);
    if let Some(mode) = mode {
        command.arg(mode);
    }
    command.output().unwrap()
}

fn attribution(output: &Output, root: &std::path::Path) -> Vec<String> {
    let mut lines = String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| {
            if line.starts_with("[cpu] ")
                || (line.starts_with("[memory] ") && line.contains(": peak_rss_kib="))
            {
                Some(line.split_once(": ").unwrap().0.to_owned())
            } else if line.starts_with("[hot] ") {
                let (pass, rest) = line.split_once(": ").unwrap();
                let (path, values) = rest.rsplit_once(" units=").unwrap();
                let (path, _) = path.rsplit_once(' ').unwrap();
                Some(format!(
                    "{pass}: {} units={values}",
                    path.replace(&root.to_string_lossy().to_string(), "<root>")
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    lines.sort();
    lines
}

#[test]
fn verbose_attribution_is_opt_in_and_preserves_reports_across_jobs() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/forbidden-dependencies-basic/fixture");
    let fixture = super::saved_fixture::materialize_path(&source);
    let root = fixture.path().canonicalize().unwrap();
    let plain = run(&root, "1", None);
    let basic = run(&root, "1", Some("--timings"));
    assert!(attribution(&plain, &root).is_empty());
    assert!(attribution(&basic, &root).is_empty());
    let mut expected = None;
    for jobs in ["1", "2"] {
        let verbose = run(&root, jobs, Some("--verbose-timings"));
        assert_eq!(verbose.status.code(), plain.status.code());
        assert_eq!(verbose.stdout, plain.stdout);
        let lines = attribution(&verbose, &root);
        let stderr = String::from_utf8_lossy(&verbose.stderr);
        for phase in ["discovery", "discovery.classify"] {
            for prefix in ["[timing]", "[cpu]", "[memory]"] {
                let label = format!("{prefix} {phase}:");
                let count = stderr
                    .lines()
                    .filter(|line| line.starts_with(&label))
                    .count();
                assert!(count <= 1, "duplicate {label}: {stderr}");
            }
        }
        let parse_ms = stderr
            .lines()
            .find_map(|line| line.strip_prefix("[timing] parse: "))
            .unwrap()
            .trim_end_matches("ms")
            .parse::<f64>()
            .unwrap();
        let file_ms = stderr
            .lines()
            .filter(|line| line.starts_with("[hot] parse: "))
            .map(|line| {
                line.split(" units=")
                    .next()
                    .unwrap()
                    .rsplit(' ')
                    .next()
                    .unwrap()
                    .trim_end_matches("ms")
                    .parse::<f64>()
                    .unwrap()
            })
            .sum::<f64>();
        assert!(
            file_ms <= parse_ms * jobs.parse::<f64>().unwrap() + 0.01,
            "{stderr}"
        );
        for pass in ["parse", "value_flow.extract", "value_flow.finalize"] {
            assert!(
                lines
                    .iter()
                    .any(|line| line.starts_with(&format!("[hot] {pass}:"))),
                "{lines:?}"
            );
            assert!(
                lines
                    .iter()
                    .filter(|line| line.starts_with(&format!("[hot] {pass}:")))
                    .count()
                    <= 10
            );
        }
        #[cfg(unix)]
        for prefix in [
            "[cpu] parse",
            "[memory] parse",
            "[cpu] check.analysis",
            "[memory] check.analysis",
        ] {
            assert!(lines.iter().any(|line| line == prefix), "{lines:?}");
        }
        if let Some(expected) = &expected {
            assert_eq!(&lines, expected);
        }
        expected = Some(lines);
    }
}

#[test]
fn annotation_hot_files_cover_the_prepared_root_evaluation() {
    let fixture = super::saved_fixture::materialize(
        "performance",
        "annotation-shared-record-dag/observer-batching",
    );
    let root = fixture.path().canonicalize().unwrap();
    let first = run(&root, "1", Some("--verbose-timings"));
    let second = run(&root, "2", Some("--verbose-timings"));
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
    let lines = attribution(&first, &root);
    #[cfg(unix)]
    {
        assert!(lines.iter().any(|line| line == "[cpu] parse"));
        assert!(lines
            .iter()
            .any(|line| line == "[cpu] postgres.annotation.project"));
        assert!(!lines.iter().any(|line| line == "[cpu] facts.prepare"));
    }
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("[hot] postgres.annotation:")),
        "{lines:?}"
    );
    assert_eq!(lines, attribution(&second, &root));
}
