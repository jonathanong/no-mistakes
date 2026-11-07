use super::*;

#[test]
fn origins_follow_getters_owned_lets_and_parameters_but_nothing_else() {
    let read = read("origins.mjs");
    let env = |needle: &str| read.origin(needle).env;
    assert_eq!(env("readdirSync(decl())").as_deref(), Some("A"));
    assert_eq!(env("readdirSync(expr())").as_deref(), Some("B"));
    assert_eq!(env("readdirSync(arrow())").as_deref(), Some("C"));
    assert_eq!(env("readdirSync(block())").as_deref(), Some("D"));
    assert_eq!(env("readdirSync(twice())").as_deref(), Some("G"));
    assert_eq!(env("readdirSync(process.env.H").as_deref(), Some("H"));
    for unknown in [
        "readdirSync(withArg(1))",
        "readdirSync(gen())",
        "readdirSync(missing())",
        "readdirSync(argArrow())",
        "readdirSync(noReturn())",
        "readdirSync(notFunction())",
        "readdirSync(\"x\" && process.env.I)",
        "readdirSync(process.env[String(1)])",
        "readdirSync(process.env.J && \"x\")",
    ] {
        assert_eq!(read.origin(unknown), PathOrigin::unknown(), "{unknown}");
    }
    assert_eq!(read.origin("readdirSync(owned)").anchor, PathAnchor::Temp);
    for unowned in [
        "readdirSync(mixed)",
        "readdirSync(compound)",
        "readdirSync(unset)",
        "readdirSync(exported)",
        "readdirSync(destructured)",
        "readdirSync(pattern)",
    ] {
        assert_eq!(read.origin(unowned), PathOrigin::unknown(), "{unowned}");
    }
    let param = |needle: &str| read.origin(needle).param;
    assert_eq!(param("readdirSync(first)"), None);
    assert_eq!(param("readdirSync(second);"), Some(1));
    assert_eq!(param("readdirSync(rest)"), None);
    assert_eq!(param("readdirSync(second)]"), Some(1));
    let owner = read.source.find("function param(").unwrap() as u32;
    assert_eq!(read.origin("readdirSync(second);").param_owner, Some(owner));
    assert_eq!(read.origin("readdirSync(second)]").param_owner, Some(owner));
    assert_eq!(param("readdirSync(a)"), None);
    assert_eq!(param("readdirSync(b)"), None);
    assert_eq!(param("readdirSync(join(c"), Some(2));
}

#[test]
fn only_owned_or_module_rooted_arguments_of_imported_calls_are_kept() {
    let read = read("origins.mjs");
    let offset = read.source.find("local(\"x\"").unwrap() as u32;
    let paths = &read.provenance().arg_paths[&offset];
    let kept: Vec<(usize, PathAnchor)> = paths
        .iter()
        .map(|(index, origin)| (*index, origin.anchor))
        .collect();
    assert_eq!(
        kept,
        [
            (1, PathAnchor::Temp),
            (3, PathAnchor::Module),
            (4, PathAnchor::Cwd)
        ]
    );
}
