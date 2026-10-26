use super::*;
use serde_json::{json, Value};

fn report(name: &str, policy: Option<&str>, include: bool) -> Value {
    let root = static_check_fixture(&format!("redirect-suppression-{name}"));
    let mut options = json!({ "root": root, "includeSuppressed": include });
    if let Some(policy) = policy {
        options["config"] = json!(root.join(format!("{policy}.yml")));
    }
    serde_json::from_str(
        &check_json_impl(crate::napi_api::options::test_json_arg(options.to_string())).unwrap(),
    )
    .unwrap()
}

#[test]
fn entry_file_directive_suppresses_delegated_findings_in_every_policy() {
    for policy in [None, Some("report"), Some("ignore")] {
        for include in [false, true] {
            let value = report("entry-disabled", policy, include);
            assert_eq!(value["rules"], json!([]), "{value}");
            assert_eq!(value["advisories"], json!([]), "{value}");
            if include {
                let records = value["suppressed"].as_array().unwrap();
                assert_eq!(records.len(), 1, "{value}");
                assert_eq!(records[0]["file"], "next-shared.ts");
                assert_eq!(records[0]["directiveKind"], "file");
                assert_eq!(records[0]["directiveLine"], 1);
            }
        }
    }
}

#[test]
fn helper_directives_and_public_helper_locations_survive_entry_fallback() {
    let control = report("control", None, true);
    assert_eq!(control["rules"].as_array().unwrap().len(), 1, "{control}");
    assert_eq!(control["rules"][0]["file"], "next-shared.ts");
    let helper = report("helper-disabled", None, true);
    assert_eq!(helper["rules"], json!([]), "{helper}");
    assert_eq!(helper["advisories"], json!([]), "{helper}");
    assert_eq!(helper["suppressed"].as_array().unwrap().len(), 1);
}

#[test]
fn entry_file_opt_out_has_priority_over_redundant_helper_comment() {
    let value = report("both-disabled", None, true);
    assert_eq!(value["rules"], json!([]), "{value}");
    let advisory = value["advisories"].as_array().unwrap();
    assert_eq!(advisory.len(), 1, "{value}");
    assert_eq!(advisory[0]["file"], "next-shared.ts");
    assert_eq!(advisory[0]["rule"], "unused-disable-directive");
    assert_eq!(value["suppressed"].as_array().unwrap().len(), 1);
}

#[test]
fn empty_delegated_config_does_not_consume_entry_file_directive() {
    for policy in [None, Some("report"), Some("ignore")] {
        let value = report("entry-stale", policy, true);
        assert_eq!(value["suppressed"], json!([]), "{value}");
        let channel = if policy == Some("report") {
            "rules"
        } else {
            "advisories"
        };
        if policy == Some("ignore") {
            assert_eq!(value[channel], json!([]), "{value}");
        } else {
            assert_eq!(value[channel].as_array().unwrap().len(), 1, "{value}");
            assert_eq!(value[channel][0]["rule"], "unused-disable-directive");
            assert_eq!(value[channel][0]["file"], "next.config.ts");
        }
    }
}

#[test]
fn entry_line_directive_never_matches_a_delegated_helpers_line() {
    let value = report("entry-line", None, true);
    assert_eq!(value["rules"].as_array().unwrap().len(), 1, "{value}");
    assert_eq!(value["rules"][0]["file"], "next-shared.ts");
    assert_eq!(value["suppressed"], json!([]));
    assert_eq!(value["advisories"][0]["rule"], "unused-disable-directive");
}

#[test]
fn entry_file_directive_accounts_for_delegated_advisories() {
    let value = report("entry-advisory", None, true);
    assert_eq!(value["rules"], json!([]), "{value}");
    assert_eq!(value["advisories"], json!([]), "{value}");
    assert!(
        !value["suppressed"].as_array().unwrap().is_empty(),
        "{value}"
    );
    assert!(value["suppressed"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["domain"] == "advisories" && item["file"] == "next-shared.ts"));
}
