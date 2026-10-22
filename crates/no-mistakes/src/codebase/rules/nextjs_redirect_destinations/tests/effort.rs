use super::super::{
    advisories_with_files_and_sources, check_with_files_and_sources, extract, RuleFinding,
};
use super::{config, fixture, fixture_files, run};
use crate::codebase::rules::analysis_policy::prepare_rule_analysis;
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};
use std::path::{Path, PathBuf};

const REWRITE: &str =
    "return a literal array of objects with string destination fields, for example return [{ source: '/a', destination: '/b', permanent: true }]";
const STATIC: &str = "configPath: web/next.config.ts\nappRoot: web/app\nincludeRewrites: false";

fn prepare(name: &str, options_yaml: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = fixture(name);
    let files = fixture_files(&root);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut config = config(options_yaml);
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    (root, config, files)
}

fn run_prepared(name: &str, options_yaml: &str) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    let (root, config, files) = prepare(name, options_yaml);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    (
        check_with_files_and_sources(&root, &config, &files, &sources).unwrap(),
        advisories_with_files_and_sources(&root, &config, &files, &sources).unwrap(),
    )
}

fn with_max(options: &str) -> String {
    if options.trim() == "{}" {
        "{ effort: max }".into()
    } else {
        format!("{options}\neffort: max")
    }
}

#[test]
fn max_matches_unprepared_output_on_saved_fixtures() {
    for (name, options) in [
        ("pass", "{}"),
        ("dest-non-string", "{}"),
        ("fail-missing", "{}"),
        ("fail-extractor", "{}"),
        ("static-tuples-pass", STATIC),
        ("static-tuples-fail", STATIC),
        ("partial-dynamic", STATIC),
    ] {
        let baseline = run(name, options);
        let (max_default, advisories) = run_prepared(name, options);
        assert_eq!(max_default, baseline, "{name} default");
        assert!(advisories.is_empty(), "{name}: {advisories:?}");
        let (explicit, _) = run_prepared(name, &with_max(options));
        assert_eq!(explicit, baseline, "{name} explicit max");
    }
}

#[test]
fn low_accepts_literal_destinations_and_still_flags_missing_pages() {
    let (findings, advisories) = run_prepared("pass", "{ effort: low }");
    assert!(findings.is_empty(), "{findings:?}");
    assert!(advisories.is_empty(), "{advisories:?}");
    let (findings, advisories) = run_prepared("fail-missing", "{ effort: low }");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("redirect destination '/gone'"));
    assert!(advisories.is_empty(), "{advisories:?}");
}

#[test]
fn low_routes_non_literal_destinations_through_unanalyzable_policy() {
    for (policy, options) in [
        (
            Unanalyzable::Report,
            "{ effort: low, unanalyzable: report }",
        ),
        (
            Unanalyzable::Advise,
            "{ effort: low, unanalyzable: advise }",
        ),
        (
            Unanalyzable::Ignore,
            "{ effort: low, unanalyzable: ignore }",
        ),
    ] {
        let (findings, advisories) = run_prepared("dest-non-string", options);
        let residuals: Vec<_> = findings.iter().chain(&advisories).collect();
        match policy {
            Unanalyzable::Report => {
                assert_eq!(findings.len(), 1, "{findings:?}");
                assert!(advisories.is_empty());
            }
            Unanalyzable::Advise => {
                assert!(findings.is_empty(), "{findings:?}");
                assert_eq!(advisories.len(), 1, "{advisories:?}");
            }
            Unanalyzable::Ignore => {
                assert!(findings.is_empty(), "{findings:?}");
                assert!(advisories.is_empty(), "{advisories:?}");
            }
        }
        for finding in residuals {
            assert!(finding.message.contains("effort low"), "{finding:?}");
            assert!(finding.message.contains(REWRITE), "{finding:?}");
        }
    }
}

#[test]
fn low_treats_map_callbacks_as_unanalyzable() {
    let (findings, advisories) = run_prepared(
        "static-tuples-pass",
        "configPath: web/next.config.ts\nappRoot: web/app\nincludeRewrites: false\neffort: low",
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].message.contains("unanalyzable"));
    assert!(findings[0].message.contains(REWRITE));
    assert!(advisories.is_empty(), "{advisories:?}");
}

#[test]
fn literals_only_skips_identifiers_and_keeps_literal_arrays() {
    let path = Path::new("next.config.ts");
    let source = r#"
const dest = "/about";
export default {
  async redirects() {
    return [{ source: "/old", destination: dest, permanent: true }];
  },
};
"#;
    let max = extract::extract_named_destinations(path, source, "redirects");
    let low = extract::extract_named_destinations_with(path, source, "redirects", true);
    assert_eq!(max.destinations[0].value, "/about");
    assert!(!max.incomplete);
    assert!(low.destinations.is_empty());
    assert!(low.saw_destination_property);
    assert!(low.incomplete);

    let literals = r#"
export default {
  async redirects() {
    return [{ source: "/old", destination: "/about", permanent: true }];
  },
};
"#;
    let low = extract::extract_named_destinations_with(path, literals, "redirects", true);
    assert_eq!(low.destinations[0].value, "/about");
    assert!(!low.incomplete);
}

#[test]
fn literals_only_rejects_spreads_helpers_templates_and_local_bindings() {
    let path = Path::new("next.config.ts");
    for source in [
        r#"export default { async redirects() { return [...rows]; } };"#,
        r#"export default { async redirects() { return [{ ...row, destination: "/about" }]; } };"#,
        r#"export default { async redirects() { return rows.map((row) => row); } };"#,
        r#"export default { async redirects() { return [{ destination: `/x/${id}` }]; } };"#,
        r#"export default { async redirects() { const dest = "/about"; return [{ destination: dest }]; } };"#,
    ] {
        let low = extract::extract_named_destinations_with(path, source, "redirects", true);
        assert!(low.body_found, "{source}");
        assert!(low.incomplete, "{source}");
    }
    let template =
        r#"export default { async redirects() { return [{ destination: `/about` }]; } };"#;
    let low = extract::extract_named_destinations_with(path, template, "redirects", true);
    assert_eq!(low.destinations[0].value, "/about");
    assert!(!low.incomplete);
}

#[test]
fn prepared_low_is_recorded_on_the_application() {
    let (_, config, _) = prepare("pass", "{ effort: low }");
    let analysis = config.effective_rule_analysis(&config.rules[0]).unwrap();
    assert_eq!(analysis.effort, Effort::Low);
    assert!(config.analysis_warnings.is_empty());
}
