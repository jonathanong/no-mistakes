use super::place_support::{argv, assert_quiet, live, owned, shell};
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;

#[test]
fn a_package_manager_directory_replaces_an_owned_cwd() {
    let message = live(&argv(
        "pnpm",
        &["-C", "/repo", "exec", "widget-audit", "check"],
        Some(owned()),
    ));
    assert!(message.contains("in `/repo`"));
    assert!(!message.contains("set `cwd`"));
    assert!(live(&argv(
        "pnpm",
        &["--dir", "/repo", "exec", "widget-audit", "check"],
        Some(owned()),
    ))
    .contains("in `/repo`"));
    assert!(live(&argv(
        "pnpm",
        &["-C/repo", "exec", "widget-audit", "check"],
        Some(owned()),
    ))
    .contains("in `/repo`"));
    assert!(live(&argv(
        "yarn",
        &["--cwd", "/repo", "widget-audit", "check"],
        Some(owned()),
    ))
    .contains("in `/repo`"));
    assert!(live(&argv(
        "yarn",
        &["--cwd=/repo", "widget-audit", "check"],
        Some(owned()),
    ))
    .contains("in `/repo`"));
    assert!(live(&argv(
        "npx",
        &["pnpm", "-C", "/repo", "exec", "widget-audit", "check"],
        Some(owned()),
    ))
    .contains("in `/repo`"));
    assert!(live(&shell(
        "cd fixtures && pnpm -C /repo exec widget-audit check",
        Some(owned()),
    ))
    .contains("in `/repo`"));
    let bare = live(&argv(
        "pnpm",
        &["-C", "/repo", "exec", "widget-audit", "check"],
        None,
    ));
    assert!(bare.contains("set `cwd`"));
    assert_quiet(&argv(
        "pnpm",
        &["-C", "fixtures", "exec", "widget-audit", "check"],
        Some(owned()),
    ));
    assert_quiet(&argv(
        "pnpm",
        &["--filter", "widgets", "exec", "widget-audit", "check"],
        Some(owned()),
    ));
    assert_quiet(&argv(
        "pnpm",
        &[
            "-C",
            "/repo",
            "--dir",
            "fixtures",
            "exec",
            "widget-audit",
            "check",
        ],
        Some(owned()),
    ));
    assert_quiet(&shell(
        "cd /repo && pnpm -C fixtures exec widget-audit check",
        Some(owned()),
    ));
}

#[test]
fn shell_dot_root_and_parent_follow_the_base_directory() {
    assert!(live(&shell("cd . && widget-audit check", None)).contains("set `cwd`"));
    assert_quiet(&shell("cd . && widget-audit check", Some(owned())));
    assert!(live(&shell("cd ././ && widget-audit check", None)).contains("set `cwd`"));
    let root = live(&shell("cd / && widget-audit check", Some(owned())));
    assert!(root.contains("after `cd /`"));
    assert!(!root.contains("set `cwd`"));
    assert!(live(&shell("cd / && widget-audit check", None)).contains("set `cwd`"));
    assert!(live(&shell("cd /./ && widget-audit check", Some(owned()))).contains("after `cd /./`"));
    assert!(live(&shell("cd .. && widget-audit check", None)).contains("set `cwd`"));
    assert!(live(&shell(
        "cd .. && widget-audit check",
        Some(PathOrigin::unknown())
    ))
    .contains("set `cwd`"));
    let parent = shell("cd .. && widget-audit check", Some(owned()));
    assert!(parent.findings.is_empty());
    assert_eq!(parent.advisories.len(), 1);
    let named = live(&shell("cd /repo && widget-audit check", Some(owned())));
    assert!(named.contains("after `cd /repo`"));
    assert!(!named.contains("set `cwd`"));
}

#[test]
fn later_simple_commands_and_quote_free_expansions() {
    assert_quiet(&shell("cd fixtures/$DIR && widget-audit check", None));
    assert_quiet(&shell("cd fixtures/`item` && widget-audit check", None));
    assert!(live(&shell("true && widget-audit check", None)).contains("set `cwd`"));
    assert!(live(&shell("true&&widget-audit check", None)).contains("set `cwd`"));
    assert!(live(&shell("\"widget-audit\" check", None)).contains("set `cwd`"));
    assert!(live(&shell("widget-au''dit check", None)).contains("set `cwd`"));
    let sequenced = live(&shell("cd widgets; widget-audit check", Some(owned())));
    assert!(sequenced.contains("after `cd widgets`"));
    let alternate = live(&shell("cd widgets || widget-audit check", None));
    assert!(alternate.contains("set `cwd`"));
    assert!(!alternate.contains("after `cd"));
    let carried = live(&shell(
        "cd fixtures && widget-audit check && cd /repo && widget-audit check",
        Some(owned()),
    ));
    assert!(carried.contains("after `cd /repo`"));
    let literal = shell("cd 'fixtures/$item' && widget-audit check", None);
    assert!(literal.findings.is_empty());
    assert_eq!(literal.advisories.len(), 1);
}

#[test]
fn an_unresolved_directory_does_not_hide_a_later_live_scan() {
    let chained = shell(
        "cd ../widgets && widget-audit check && cd /repo && widget-audit check",
        None,
    );
    assert_eq!(chained.advisories.len(), 1);
    assert!(chained.advisories[0].message.contains("unresolved root"));
    assert_eq!(chained.findings.len(), 1);
    assert!(chained.findings[0].message.contains("live repository scan"));
}
