use super::*;

const WORKFLOW: &str = ".github/workflows/coverage.yml";

fn fixture_source() -> String {
    std::fs::read_to_string(fixture_root("effort-argv").join(WORKFLOW)).unwrap()
}

fn script(name: &str) -> String {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&fixture_source()).unwrap();
    workflow["jobs"]["verify"]["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"].as_str() == Some(name))
        .unwrap()["run"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn alias_identity_proofs_do_not_broaden_download_target_scope() {
    for name in [
        "downloader-alias-without-http-url",
        "unrelated-literal-binding",
        "separator-only-command-prefix",
        "multiple-non-downloader-commands",
    ] {
        let script = script(name);
        assert!(judge::download_hits(&script, &[]).is_empty(), "max: {name}");
        assert!(
            judge::download_hits_low(&script, &[]).is_empty(),
            "low: {name}"
        );
    }
}

#[test]
fn literal_url_positions_survive_option_end_markers_and_ordinary_short_flags() {
    for name in ["explicit-end-of-options", "ordinary-short-option-cluster"] {
        let script = script(name);
        let max = judge::download_hits(&script, &[]);
        let low = judge::download_hits_low(&script, &[]);
        assert_eq!(max.len(), 1, "{name}");
        assert_eq!(low.len(), 1, "{name}");
        assert!(!low[0].unanalyzable, "{name}");
        assert_eq!(low[0].url, max[0].url, "{name}");
        assert!(judge::filter_trusted(low, &hosts()).is_empty(), "{name}");
    }
}

#[test]
fn dynamic_short_option_names_route_uncertainty_through_each_policy() {
    let script = script("dynamic-short-option-name");
    // Max proves that -$FLAG becomes -o and consumes the following filename.
    assert!(judge::download_hits(&script, &[]).is_empty());
    let low = judge::download_hits_low(&script, &[]);
    assert_eq!(low.len(), 1);
    assert!(low[0].unanalyzable);
    assert_eq!(low[0].url, "-$FLAG");

    let source = fixture_source();
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let result = runs::check_source_low(WORKFLOW, &source, &hosts(), &[], policy);
        assert_eq!(
            result.findings.len(),
            usize::from(policy == Unanalyzable::Report)
        );
        assert_eq!(
            result.advisories.len(),
            usize::from(policy == Unanalyzable::Advise)
        );
        for finding in result.findings.iter().chain(&result.advisories) {
            assert!(finding.message.contains("-$FLAG"));
            assert!(finding.message.contains("unanalyzable at effort low"));
            assert!(finding.message.contains("literal option names"));
        }
    }
}
