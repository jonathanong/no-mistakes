use super::*;

#[test]
fn object_arguments_require_small_plain_static_properties() {
    let read = read("member-arguments.mjs");
    let members = &read.provenance().arg_member_paths;
    let accepted = read.source.find("inspect({ z:").unwrap() as u32;
    let paths = &members[&accepted];
    assert_eq!(
        paths
            .iter()
            .map(|(index, key, origin)| (*index, key.as_str(), origin.anchor))
            .collect::<Vec<_>>(),
        [(0, "a", PathAnchor::Temp), (0, "z", PathAnchor::Temp)]
    );
    assert_eq!(members.len(), 1);
    for rejected in [
        "inspect({ root, ...",
        "inspect({ [key]",
        "inspect({ get root",
        "inspect({ method",
        "inspect({ root: root",
        "inspect({ k0:",
        "inspect(...args)",
    ] {
        let offset = read.source.find(rejected).unwrap() as u32;
        assert!(!members.contains_key(&offset), "{rejected}");
    }
}

#[test]
fn getter_setups_require_owned_complete_return_paths() {
    let read = read("getter-rejected-statements.mjs");
    assert_eq!(read.origin("readdirSync(tail())"), PathOrigin::unknown());
    for accepted in ["readdirSync(branch())", "readdirSync(accepted())"] {
        assert_eq!(read.origin(accepted).anchor, PathAnchor::Temp, "{accepted}");
    }
}
