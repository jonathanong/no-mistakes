use super::*;

const OPERANDS: &str = ".github/workflows/dynamic-option-operands.yml";

fn operand_source() -> String {
    std::fs::read_to_string(fixture_root("effort-argv").join(OPERANDS)).unwrap()
}

fn operand_script(name: &str) -> String {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&operand_source()).unwrap();
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
fn only_quoted_option_values_preserve_the_following_url_slot() {
    for (unquoted, quoted) in [
        ("unquoted-output-operand", "quoted-output-operand"),
        ("unquoted-attached-output", "quoted-attached-output"),
        ("unquoted-inline-output", "quoted-inline-output"),
    ] {
        let script = operand_script(unquoted);
        assert!(judge::download_hits(&script, &[]).is_empty(), "{unquoted}");
        let low = judge::download_hits_low(&script, &[]);
        assert_eq!(low.len(), 1, "{unquoted}");
        assert!(low[0].unanalyzable, "{unquoted}");

        let script = operand_script(quoted);
        let max = judge::download_hits(&script, &[]);
        let low = judge::download_hits_low(&script, &[]);
        assert_eq!(max.len(), 1, "{quoted}");
        assert_eq!(low.len(), 1, "{quoted}");
        assert!(!low[0].unanalyzable, "{quoted}");
        assert_eq!(low[0].url, max[0].url, "{quoted}");
    }
}

#[test]
fn uncertain_option_values_follow_policy_without_dropping_literal_violations() {
    let source = operand_source();
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let result = runs::check_source_low(OPERANDS, &source, &hosts(), &[], policy);
        assert_eq!(
            result.findings.len(),
            if policy == Unanalyzable::Report { 6 } else { 3 }
        );
        assert_eq!(
            result.advisories.len(),
            if policy == Unanalyzable::Advise { 3 } else { 0 }
        );
        let shapes: Vec<_> = result
            .findings
            .iter()
            .chain(&result.advisories)
            .filter(|finding| finding.message.contains("unanalyzable at effort low"))
            .collect();
        assert_eq!(
            shapes.len(),
            if policy == Unanalyzable::Ignore { 0 } else { 3 }
        );
        for shape in shapes {
            assert!(shape.message.contains("$OUT"));
            assert!(!shape.message.contains("does not verify"));
        }
    }
}
