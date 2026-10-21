use super::*;
use std::path::{Path, PathBuf};

#[test]
fn local_use_paths_map_the_repository_root_and_reject_parents() {
    assert_eq!(local_use_relative("actions/checkout@v4"), None);
    assert_eq!(local_use_relative("./"), Some(String::new()));
    assert_eq!(local_use_relative("./."), Some(String::new()));
    assert_eq!(local_use_relative("././"), Some(String::new()));
    assert_eq!(local_use_relative("./../secret/action.yml"), None);
    assert_eq!(local_use_relative("./foo/../../secret/action.yml"), None);
    assert_eq!(
        local_use_relative(r".\actions\outer"),
        Some("actions/outer".to_owned())
    );
    // `.` and empty segments collapse. In-repo `..` resolves to the same descriptor.
    assert_eq!(
        local_use_relative("./.github/actions/./deploy"),
        Some(".github/actions/deploy".to_owned())
    );
    assert_eq!(
        local_use_relative("./.github/actions//deploy"),
        Some(".github/actions/deploy".to_owned())
    );
    assert_eq!(
        local_use_relative("./.github/actions/deploy/../deploy"),
        Some(".github/actions/deploy".to_owned())
    );
}

#[test]
fn descriptor_index_ignores_paths_without_file_names() {
    assert!(action_descriptors(Path::new("/repo"), [PathBuf::from("/")]).is_empty());
}

#[test]
fn tagged_and_non_string_uses_do_not_invent_local_actions() {
    use serde_yaml::value::{Tag, TaggedValue};

    let mut mapping = serde_yaml::Mapping::new();
    mapping.insert(
        Value::String("uses".into()),
        Value::String("  ./actions/outer  ".into()),
    );
    let tagged = Value::Tagged(Box::new(TaggedValue {
        tag: Tag::new("!relative"),
        value: Value::Mapping(mapping),
    }));
    let mut uses = Vec::new();
    collect_local_uses(&tagged, &mut uses);
    assert_eq!(uses, vec!["./actions/outer".to_owned()]);

    let mut ignored = serde_yaml::Mapping::new();
    ignored.insert(Value::String("uses".into()), Value::Sequence(Vec::new()));
    uses.clear();
    collect_local_uses(&Value::Mapping(ignored), &mut uses);
    assert!(uses.is_empty());
}

#[test]
fn descriptors_follow_directories_and_ignore_workflow_files() {
    assert_eq!(descriptor_relatives(""), ["action.yml", "action.yaml"]);
    assert_eq!(descriptor_relatives("."), ["action.yml", "action.yaml"]);
    assert_eq!(
        descriptor_relatives("actions/outer"),
        ["actions/outer/action.yml", "actions/outer/action.yaml"]
    );
    assert_eq!(
        descriptor_relatives("actions/outer/action.yml"),
        ["actions/outer/action.yml"]
    );
    assert_eq!(descriptor_relatives("action.yaml"), ["action.yaml"]);
    assert!(descriptor_relatives(".github/workflows/reusable.yml").is_empty());
    assert!(descriptor_relatives(".github/workflows/reusable.yaml").is_empty());
}
