use super::*;
use crate::cli::Format;
use crate::codebase::queries::render::render;
use std::path::PathBuf;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn args(root: PathBuf, file: &str, export: &str) -> CallSitesArgs {
    CallSitesArgs {
        file: PathBuf::from(file),
        export_name: export.to_string(),
        root: Some(root),
        tsconfig: None,
        format: None,
        json: false,
    }
}

#[test]
fn collects_sites_with_callers_and_spread() {
    let report = compute(&args(fixture_root("queries"), "util.ts", "used")).unwrap();
    assert_eq!(report.call_sites.len(), 4);
    // Sorted by (file, line): broken.ts then three in consumer.ts.
    assert_eq!(report.call_sites[0].file, "broken.ts");
    assert_eq!(report.call_sites[0].caller.as_deref(), Some("broken"));

    let spread = report
        .call_sites
        .iter()
        .find(|site| site.has_spread)
        .unwrap();
    assert_eq!(spread.arg_count, 1);
    assert_eq!(spread.args, vec!["spread"]);

    // The top-level call has no enclosing function.
    assert!(report.call_sites.iter().any(|site| site.caller.is_none()));
}

#[test]
fn covers_every_argument_shape() {
    let json = run_json(args(fixture_root("queries-shapes"), "target.ts", "f")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let arg_lists: Vec<Vec<String>> = value["callSites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| {
            site["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a.as_str().unwrap().to_string())
                .collect()
        })
        .collect();
    assert!(arg_lists.contains(&vec!["string".into(), "string".into()]));
    assert!(arg_lists.contains(&vec!["number".into(), "number".into()]));
    assert!(arg_lists.contains(&vec!["boolean".into(), "null".into()]));
    assert!(arg_lists.contains(&vec!["identifier".into(), "object".into()]));
    assert!(arg_lists.contains(&vec!["array".into(), "arrow".into()]));
    assert!(arg_lists.contains(&vec!["call".into(), "arrow".into()]));
    assert!(arg_lists.contains(&vec!["other".into()]));
}

#[test]
fn renders_formats_and_runs() {
    let report = compute(&args(fixture_root("queries"), "util.ts", "used")).unwrap();
    let mut human = Vec::new();
    render(&report, Format::Human, &mut human).unwrap();
    let text = String::from_utf8(human).unwrap();
    assert!(text.contains("util.ts#used"));
    assert!(text.contains("(top-level)"));
    assert!(text.contains("run(string, object)"));

    let mut paths = Vec::new();
    render(&report, Format::Paths, &mut paths).unwrap();
    assert!(String::from_utf8(paths).unwrap().contains("consumer.ts"));

    for format in [Format::Json, Format::Yml, Format::Md] {
        let mut buf = Vec::new();
        render(&report, format, &mut buf).unwrap();
        assert!(!buf.is_empty());
    }
    let _ = run(args(fixture_root("queries"), "util.ts", "used")).unwrap();
}
