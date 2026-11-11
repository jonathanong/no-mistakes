use super::EnvWrites;
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};
use crate::fx::FxHashMap;

#[test]
fn tmpdir_primary_environment_is_platform_specific() {
    for (windows, primary, other) in [(true, "TEMP", "TMPDIR"), (false, "TMPDIR", "TEMP")] {
        let origin = PathOrigin::tmpdir(windows);
        assert_eq!(origin.anchor, PathAnchor::SharedTemp);
        assert_eq!(origin.env.as_deref(), Some(primary));
        let opposite_only = EnvWrites {
            owned: FxHashMap::from_iter([(other.to_string(), true)]),
            ..EnvWrites::default()
        };
        assert!(!opposite_only.all_owned(origin.env.as_deref().unwrap()));
        let primary_owned = EnvWrites {
            owned: FxHashMap::from_iter([(primary.to_string(), true)]),
            ..EnvWrites::default()
        };
        assert!(primary_owned.all_owned(origin.env.as_deref().unwrap()));
        let fallback_only = EnvWrites {
            owned: FxHashMap::from_iter([("TMP".to_string(), true)]),
            ..EnvWrites::default()
        };
        assert!(!fallback_only.all_owned(origin.env.as_deref().unwrap()));
    }
}
