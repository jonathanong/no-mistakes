use super::*;

mod coverage;
mod options;

fn argv_script(name: &str) -> String {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&source("effort-argv")).unwrap();
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
fn low_proves_downloader_alias_scope_without_expanding_url_arguments() {
    let helpers = ["ci_download_to".to_string()];
    for name in [
        "literal-assignment-command",
        "prefixed-command",
        "configured-helper-command",
    ] {
        let script = argv_script(name);
        let before = assign::probe::count();
        let binding_reads = assign::probe::binding_scans();
        let low = test_adapters::download_hits_low(&script, &helpers);
        assert_eq!(assign::probe::count(), before, "{name}");
        assert_eq!(assign::probe::binding_scans(), binding_reads + 1, "{name}");
        assert_eq!(low.len(), 1, "{name}");
        assert!(low[0].unanalyzable && low[0].indirect_command, "{name}");
        let max = test_adapters::download_hits(&script, &helpers);
        assert_eq!(max.len(), 1, "{name}");
        assert!(!max[0].unanalyzable, "{name}");
    }
    assert!(
        test_adapters::download_hits_low(&argv_script("non-downloader-command"), &helpers)
            .is_empty()
    );
}

#[test]
fn low_does_not_claim_unsafe_urls_when_dynamic_arguments_can_change_url_slots() {
    for name in [
        "dynamic-output-flags",
        "static-host-dynamic-path",
        "mixed-literal-and-dynamic-url-command",
    ] {
        let low = test_adapters::download_hits_low(&argv_script(name), &[]);
        assert_eq!(low.len(), 1, "{name}");
        assert!(low.iter().all(|hit| hit.unanalyzable), "{name}");
    }
    // At max this argument becomes --output, making the URL an output filename.
    assert!(test_adapters::download_hits(&argv_script("dynamic-output-flags"), &[]).is_empty());
}

#[test]
fn low_keeps_non_url_option_operands_out_of_url_diagnostics() {
    for name in [
        "dynamic-output-operand",
        "dynamic-header-operand",
        "dynamic-inline-option-operands",
        "dynamic-attached-short-operands",
        "dynamic-cluster-option-operand",
        "dynamic-option-looking-header-operand",
        "dynamic-wget-output-operand",
    ] {
        let script = argv_script(name);
        let low = judge::filter_trusted(test_adapters::download_hits_low(&script, &[]), &hosts());
        assert!(low.is_empty(), "{name}");
    }
}

#[test]
fn non_ascii_non_url_operands_do_not_panic_during_prefix_classification() {
    let script = argv_script("unicode-non-url-operand");
    assert!(test_adapters::download_hits(&script, &[]).is_empty());
    assert!(test_adapters::download_hits_low(&script, &[]).is_empty());
}

#[test]
fn low_retains_downloader_scope_after_an_unquoted_empty_wrapper() {
    let script = argv_script("elided-wrapper-command");
    let max = test_adapters::download_hits(&script, &[]);
    assert_eq!(max.len(), 1);
    let before = assign::probe::count();
    let low = test_adapters::download_hits_low(&script, &[]);
    assert_eq!(assign::probe::count(), before);
    assert_eq!(low.len(), 1);
    assert!(low[0].unanalyzable && low[0].indirect_command);
}

#[test]
fn command_aliases_do_not_supply_shell_separators_or_redirections() {
    for name in ["literal-command-separators", "heredoc-command-prefix"] {
        let script = argv_script(name);
        assert!(
            test_adapters::download_hits(&script, &[]).is_empty(),
            "{name}"
        );
        assert!(
            test_adapters::download_hits_low(&script, &[]).is_empty(),
            "{name}"
        );
    }
}

#[test]
fn dynamic_option_names_use_the_selected_uncertainty_policy() {
    let workflow = ".github/workflows/dynamic-option-name.yml";
    let source = std::fs::read_to_string(fixture_root("effort-argv").join(workflow)).unwrap();
    // Max expands --$FLAG to --output, making the following token a filename.
    assert!(test_adapters::check_source(workflow, &source, &hosts(), &[]).is_empty());
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let result = test_adapters::check_source_low(workflow, &source, &hosts(), &[], policy);
        assert_eq!(
            result.findings.len(),
            usize::from(policy == Unanalyzable::Report)
        );
        assert_eq!(
            result.advisories.len(),
            usize::from(policy == Unanalyzable::Advise)
        );
        for finding in result.findings.iter().chain(&result.advisories) {
            assert!(finding.message.contains("unanalyzable at effort low"));
            assert!(finding.message.contains("--$FLAG"));
            assert!(finding.message.contains("literal option names"));
            assert!(!finding.message.contains("does not verify"));
        }
    }
}
