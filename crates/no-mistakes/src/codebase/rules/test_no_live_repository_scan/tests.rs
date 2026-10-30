use super::config::{Enumerator, Options, DEFAULT_FIXTURE_DIRS};
use super::enumerators::{enumerates, spied_enumerator};
use super::provenance::{classify, Verdict};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};

fn origin(anchor: PathAnchor, segments: &[&str]) -> PathOrigin {
    PathOrigin {
        anchor,
        segments: segments.iter().map(|s| s.to_string()).collect(),
        dynamic: false,
        env: None,
        param: None,
    }
}

#[test]
fn classify_separates_bounded_live_and_unresolved_roots() {
    let dirs = DEFAULT_FIXTURE_DIRS;
    assert!(matches!(classify(None, dirs), Verdict::Unresolved(_)));
    let temp = origin(PathAnchor::Temp, &["x"]);
    assert_eq!(classify(Some(&temp), dirs), Verdict::Allowed);
    assert!(matches!(
        classify(Some(&origin(PathAnchor::SharedTemp, &[])), dirs),
        Verdict::Live(_)
    ));
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
    assert!(enumerates("fs-extra", "default.readdirSync", &[]));
    assert!(!enumerates("fs-extra", "copy", &[]));
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
    // A mocked method counts when it is built in or a configured export name.
    assert!(spied_enumerator("readdirSync", &[]));
    assert!(spied_enumerator("globSync", &[]));
    assert!(spied_enumerator("cpSync", &extra));
    assert!(!spied_enumerator("existsSync", &extra));
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
        "runtimeAssets: [{root: '', reason: why}]",
        "runtimeAssets: [{root: ./, reason: why}]",
        "runtimeAssets: [{root: /abs, reason: why}]",
        "runtimeAssets: [{root: 'a\\\\b', reason: why}]",
        "runtimeAssets: [{root: a/../b, reason: why}]",
        "runtimeAssets: [{root: a, reason: ' '}]",
    ] {
        let options: Options = serde_yaml::from_str(bad).unwrap();
        assert!(options.validate().is_err(), "{bad}");
    }
    assert!(serde_yaml::from_str::<Options>("unknown: true").is_err());
}

fn assets() -> Vec<super::config::RuntimeAsset> {
    let options: Options =
        serde_yaml::from_str("runtimeAssets: [{root: ./db/migrations/, reason: shipped}]").unwrap();
    options.validate().unwrap();
    options.runtime_assets
}

#[test]
fn runtime_assets_match_whole_segments_of_literal_paths_only() {
    use super::assets::declared;
    use std::path::Path;
    let assets = assets();
    let root = Path::new("/repo");
    let file = Path::new("/repo/src/status.ts");
    let at = |anchor, segments: &[&str]| declared(&origin(anchor, segments), root, file, &assets);
    assert!(at(PathAnchor::Module, &["..", "db", "migrations"]).is_some());
    assert!(at(PathAnchor::Module, &["..", "db", "migrations", "sub"]).is_some());
    assert!(at(PathAnchor::Cwd, &["db", "migrations"]).is_some());
    assert!(at(PathAnchor::Relative, &["db", "migrations"]).is_some());
    // Near misses, the parent, and a climb out of the repository are not assets.
    assert!(at(PathAnchor::Module, &["..", "db", "migrations-src"]).is_none());
    assert!(at(PathAnchor::Module, &["..", "db"]).is_none());
    assert!(at(PathAnchor::Module, &["..", "..", "..", "db", "migrations"]).is_none());
    assert!(at(PathAnchor::Absolute, &["db", "migrations"]).is_none());
    assert!(at(PathAnchor::Temp, &["db", "migrations"]).is_none());
    let mut dynamic = origin(PathAnchor::Cwd, &["db", "migrations"]);
    dynamic.dynamic = true;
    assert!(declared(&dynamic, root, file, &assets).is_none());
    let mut env = origin(PathAnchor::Cwd, &["db", "migrations"]);
    env.env = Some("X".into());
    assert!(declared(&env, root, file, &assets).is_none());
    // A file outside the invocation root cannot anchor a module-relative path.
    let outside = Path::new("/elsewhere/status.ts");
    assert!(declared(
        &origin(PathAnchor::Module, &["db", "migrations"]),
        root,
        outside,
        &assets
    )
    .is_none());
}

#[test]
fn only_unconditional_owned_writes_that_precede_the_call_dominate_it() {
    use super::owned::env::dominated;
    use crate::codebase::ts_source::module_facts::arguments::{
        EnvWrite, PathProvenance, WriteScope,
    };
    let write = |name: &str, anchor, scope, end| EnvWrite {
        name: name.into(),
        end,
        origin: origin(anchor, &[]),
        scope,
    };
    let provenance = PathProvenance {
        env_writes: vec![
            write("TOP", PathAnchor::Temp, WriteScope::Program, 10),
            write("LIVE", PathAnchor::Relative, WriteScope::Program, 10),
            write("BODY", PathAnchor::Temp, WriteScope::Function(20, 60), 30),
            write("HOOK", PathAnchor::Temp, WriteScope::Hook(0, 200), 50),
            write("COND", PathAnchor::Temp, WriteScope::Conditional, 10),
        ],
        test_callbacks: vec![(100, 150)],
        ..PathProvenance::default()
    };
    assert!(dominated(&provenance, "TOP", 10));
    assert!(!dominated(&provenance, "TOP", 9));
    assert!(!dominated(&provenance, "LIVE", 90));
    assert!(!dominated(&provenance, "COND", 90));
    assert!(!dominated(&provenance, "OTHER", 90));
    assert!(dominated(&provenance, "BODY", 40));
    assert!(!dominated(&provenance, "BODY", 25));
    assert!(!dominated(&provenance, "BODY", 70));
    // A hook covers test callbacks in its scope, not the registering scope itself.
    assert!(dominated(&provenance, "HOOK", 120));
    assert!(!dominated(&provenance, "HOOK", 160));
    assert!(!dominated(&provenance, "HOOK", 70));
    let mut climb = write("CLIMB", PathAnchor::Temp, WriteScope::Program, 10);
    climb.origin = origin(PathAnchor::Temp, &[".."]);
    let mut climbed = provenance.clone();
    climbed.env_writes.push(climb);
    assert!(!dominated(&climbed, "CLIMB", 90));
}

#[test]
fn json_workspace_unstub_envs_poisons_when_the_key_cannot_be_ruled_out() {
    use super::owned::env::json_text_poisons;
    assert!(!json_text_poisons(
        r#"[{ "test": { "include": ["tests/**/*.test.ts"] } }]"#
    ));
    assert!(json_text_poisons(
        r#"[{ "test": { "include": ["tests/**/*.test.ts"], "unstubEnvs": false } }]"#
    ));
    assert!(json_text_poisons(
        "// restored between cases\n[{ \"test\": { \"unstubEnvs\": true } }]"
    ));
    assert!(json_text_poisons("{"));
    assert!(!json_text_poisons(""));
}
