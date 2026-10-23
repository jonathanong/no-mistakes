use super::*;
use crate::config::v2::schema::{ProjectWaitScopes, QueueFactoryRoles};

fn shared_root_roles(config_name: &str) -> QueueFactoryRoles {
    let root = fixture("wait-scopes-shared-root");
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
    config
        .queues
        .factories
        .clone()
        .with_project_wait_scopes(&config)
}

#[test]
fn projects_sharing_a_root_check_the_union_of_wait_scopes() {
    let root = fixture("wait-scopes-shared-root");
    let roles = shared_root_roles(".no-mistakes.yml");
    assert_eq!(
        roles.wait_scopes,
        vec![ProjectWaitScopes {
            root: "api".into(),
            scopes: vec!["api/admin/**".into(), "api/routes/**".into()],
        }]
    );

    let report = analyze_project(&root, None, &[]).unwrap();
    let mut waits = report
        .check
        .iter()
        .filter(|finding| finding.kind == "enqueue-and-wait")
        .map(|finding| finding.file.replace('\\', "/"))
        .collect::<Vec<_>>();
    waits.sort();
    assert_eq!(waits, ["api/admin/send.mts", "api/routes/send.mts"]);
}

#[test]
fn merging_wait_scopes_keeps_distinct_roots_and_deduplicates_shared_globs() {
    let mut roles = shared_root_roles(".no-mistakes.yml");
    roles.merge(&shared_root_roles("merge.yml")).unwrap();
    assert_eq!(
        roles.wait_scopes,
        vec![
            ProjectWaitScopes {
                root: "api".into(),
                scopes: vec![
                    "api/admin/**".into(),
                    "api/extra/**".into(),
                    "api/routes/**".into(),
                ],
            },
            ProjectWaitScopes {
                root: "workers".into(),
                scopes: vec!["workers/tasks/**".into()],
            },
        ]
    );
}

#[test]
fn unrestricted_projects_absorb_narrow_scopes_when_attached_or_merged() {
    // An empty scope list means every file, so it must win in either order.
    for config_name in ["unrestricted-first.yml", "unrestricted-last.yml"] {
        let unrestricted = shared_root_roles(config_name);
        let expected = vec![ProjectWaitScopes {
            root: "api".into(),
            scopes: vec![],
        }];
        assert_eq!(unrestricted.wait_scopes, expected, "{config_name}");

        let narrow = shared_root_roles(".no-mistakes.yml");
        for (mut left, right) in [
            (narrow.clone(), unrestricted.clone()),
            (unrestricted.clone(), narrow),
        ] {
            left.merge(&right).unwrap();
            assert_eq!(left.wait_scopes, expected, "{config_name}");
        }
    }
}

#[test]
fn a_nested_directory_named_like_a_project_root_keeps_its_own_scope() {
    let root = fixture("wait-scopes-nested-root");
    let report = analyze_project(&root, None, &[]).unwrap();
    let waits = report
        .check
        .iter()
        .filter(|finding| finding.kind == "enqueue-and-wait")
        .map(|finding| finding.file.clone())
        .collect::<Vec<_>>();
    assert!(
        waits.iter().any(|file| file.ends_with("api/routes/r.mts")),
        "the api routes file stays in scope, waits: {waits:?}"
    );
    assert!(
        waits.iter().any(|file| file.ends_with("lib/api/util.mts")),
        "lib/api belongs to the root project, which has no waitScopes, waits: {waits:?}"
    );
    assert!(
        waits.iter().all(|file| !file.ends_with("api/other/o.mts")),
        "api/other is outside the api project's routes scope, waits: {waits:?}"
    );
}
