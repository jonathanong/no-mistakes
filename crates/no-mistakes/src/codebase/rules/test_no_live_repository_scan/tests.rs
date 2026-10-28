use super::config::{Enumerator, Options, DEFAULT_FIXTURE_DIRS};
use super::enumerators::enumerates;
use super::provenance::{classify, Verdict};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};

fn origin(anchor: PathAnchor, segments: &[&str]) -> PathOrigin {
    PathOrigin {
        anchor,
        segments: segments.iter().map(|s| s.to_string()).collect(),
        dynamic: false,
    }
}

#[test]
fn classify_separates_bounded_live_and_unresolved_roots() {
    let dirs = DEFAULT_FIXTURE_DIRS;
    assert!(matches!(classify(None, dirs), Verdict::Unresolved(_)));
    let temp = origin(PathAnchor::Temp, &["x"]);
    assert_eq!(classify(Some(&temp), dirs), Verdict::Allowed);
    let fixture = origin(PathAnchor::Module, &["fixtures", "a"]);
    assert_eq!(classify(Some(&fixture), dirs), Verdict::Allowed);
    for anchor in [
        PathAnchor::Module,
        PathAnchor::Cwd,
        PathAnchor::Relative,
        PathAnchor::Absolute,
    ] {
        let live = origin(anchor, &["src"]);
        assert!(matches!(classify(Some(&live), dirs), Verdict::Live(_)));
    }
    let unknown = PathOrigin::unknown();
    assert!(matches!(
        classify(Some(&unknown), dirs),
        Verdict::Unresolved(_)
    ));
}

#[test]
fn enumerates_matches_builtins_and_configured_imports() {
    assert!(enumerates("node:fs", "readdirSync", &[]));
    assert!(enumerates("fs", "default.readdirSync", &[]));
    assert!(enumerates("fs", "promises.readdir", &[]));
    assert!(enumerates("fs/promises", "readdir", &[]));
    assert!(enumerates("fast-glob", "default", &[]));
    assert!(!enumerates("fs", "readFileSync", &[]));
    assert!(!enumerates("globby", "isDynamicPattern", &[]));
    let extra = [
        Enumerator {
            module: "node:fs".into(),
            export: Some("cpSync".into()),
        },
        Enumerator {
            module: "walker".into(),
            export: None,
        },
    ];
    assert!(enumerates("fs", "cpSync", &extra));
    assert!(enumerates("walker", "anything", &extra));
    assert!(!enumerates("fs", "writeFileSync", &extra));
}

#[test]
fn options_validate_and_default_fixture_dirs() {
    let valid: Options = serde_yaml::from_str("fixtureDirs: [golden]").unwrap();
    valid.validate().unwrap();
    assert_eq!(valid.fixture_dirs(), ["golden"]);
    assert_eq!(Options::default().fixture_dirs(), DEFAULT_FIXTURE_DIRS);
    for bad in [
        "fixtureDirs: []",
        "fixtureDirs: ['a/b']",
        "enumerators: [{module: ' '}]",
        "enumerators: [{module: fs, export: ''}]",
    ] {
        let options: Options = serde_yaml::from_str(bad).unwrap();
        assert!(options.validate().is_err(), "{bad}");
    }
    assert!(serde_yaml::from_str::<Options>("unknown: true").is_err());
}
