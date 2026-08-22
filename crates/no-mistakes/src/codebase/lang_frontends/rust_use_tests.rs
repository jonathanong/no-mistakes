use super::rust_use::{expand_rust_use, qualify_rust_use, rust_path_prefixes};

#[test]
fn qualify_rust_use_covers_self_and_super_without_a_module() {
    assert_eq!(qualify_rust_use("self", "Item", None), "Item");
    assert_eq!(
        qualify_rust_use("self", "Item", Some("crate.mod")),
        "crate.mod.Item"
    );
    assert_eq!(qualify_rust_use("super", "Item", None), "Item");
    assert_eq!(
        qualify_rust_use("super", "Item", Some("crate.child")),
        "crate.Item"
    );
    assert_eq!(qualify_rust_use("crate", "Item", Some("ignored")), "Item");
}

#[test]
fn expand_rust_use_covers_grouped_self_empty_and_unclosed_shapes() {
    assert_eq!(expand_rust_use("foo::Bar"), vec!["foo::Bar".to_string()]);
    assert_eq!(expand_rust_use("foo::{Bar"), vec!["foo::{Bar".to_string()]);
    assert_eq!(expand_rust_use("{self}"), Vec::<String>::new());
    assert_eq!(expand_rust_use("foo::{self}"), vec!["foo".to_string()]);
    assert_eq!(expand_rust_use("foo::{ as Alias}"), vec!["foo".to_string()]);
    assert_eq!(
        expand_rust_use("foo::{Bar as Renamed}"),
        vec!["foo::Bar".to_string()]
    );
    let nested = expand_rust_use("foo::{bar::{Baz, Qux}, self}");
    assert!(nested.contains(&"foo::bar::Baz".to_string()), "{nested:?}");
    assert!(nested.contains(&"foo::bar::Qux".to_string()), "{nested:?}");
    assert!(nested.contains(&"foo".to_string()), "{nested:?}");
    assert_eq!(rust_path_prefixes("foo.bar.baz"), vec!["foo", "foo.bar"]);
}
