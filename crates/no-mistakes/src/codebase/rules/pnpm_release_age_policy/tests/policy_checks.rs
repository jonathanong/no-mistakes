use super::super::policy;
use super::*;
use std::collections::HashSet;

#[test]
fn policy_checks_report_each_cross_file_drift_kind() {
    let opts = Options {
        permanent_packages: vec![
            super::PermanentPackage {
                name: "@acme/covered".to_string(),
                reason: "first-party".to_string(),
            },
            super::PermanentPackage {
                name: "missing-permanent".to_string(),
                reason: "first-party".to_string(),
            },
        ],
        temporary_selectors: vec![
            "temporary-present@1.0.0".to_string(),
            "temporary-missing@2.0.0".to_string(),
        ],
        scoped_prefixes: vec!["@acme/".to_string()],
        ..Default::default()
    };
    let snapshot = policy::Snapshot {
        exclude: vec![
            policy::ExcludeEntry::Name("@acme/covered".to_string()),
            policy::ExcludeEntry::Name("temporary-present@1.0.0".to_string()),
            policy::ExcludeEntry::Name("unknown-package".to_string()),
            policy::ExcludeEntry::Name("unknown-package".to_string()),
            policy::ExcludeEntry::Other,
        ],
        cooldown: Some(vec![
            policy::CooldownEntry::Pattern("@acme/**".to_string()),
            policy::CooldownEntry::Other,
        ]),
        active_names: HashSet::from(["@acme/covered".to_string(), "@acme/new-tool".to_string()]),
        lockfile_keys: Some(vec!["temporary-present@1.0.0".to_string()]),
    };

    let messages = policy::check_with_complete_names(&opts, &snapshot, true)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    let body = messages.join("\n");
    assert!(body.contains("unknown-package\" duplicates"), "{body}");
    assert!(
        body.contains("unknown-package\" is not in a release-age exemption registry"),
        "{body}"
    );
    assert!(
        body.contains("missing-permanent\" is missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(
        body.contains("temporary-missing@2.0.0\" is missing from minimumReleaseAgeExclude"),
        "{body}"
    );
    assert!(
        body.contains("cooldown.exclude\" must be a string glob pattern"),
        "{body}"
    );
    assert!(
        body.contains("missing-permanent\" is not covered by npm cooldown.exclude"),
        "{body}"
    );
    assert!(
        body.contains(
            "@acme/new-tool\" is an active first-party package missing from permanentPackages"
        ),
        "{body}"
    );
    assert!(
        body.contains(
            "missing-permanent\" is registered but absent from package manifests and the lockfile"
        ),
        "{body}"
    );
    assert!(
        body.contains("temporary-missing@2.0.0\" is absent from lockfile packages"),
        "{body}"
    );
}

#[test]
fn invalid_dependabot_globs_do_not_cover_permanent_packages() {
    let opts = Options {
        permanent_packages: vec![super::PermanentPackage {
            name: "acme-lib".to_string(),
            reason: "first-party".to_string(),
        }],
        ..Default::default()
    };
    let snapshot = policy::Snapshot {
        exclude: vec![policy::ExcludeEntry::Name("acme-lib".to_string())],
        cooldown: Some(vec![policy::CooldownEntry::Pattern("[".to_string())]),
        active_names: HashSet::from(["acme-lib".to_string()]),
        lockfile_keys: None,
    };

    let messages = policy::check_with_complete_names(&opts, &snapshot, true)
        .into_iter()
        .map(|issue| issue.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("acme-lib\" is not covered by npm cooldown.exclude")),
        "{messages:#?}"
    );
}
