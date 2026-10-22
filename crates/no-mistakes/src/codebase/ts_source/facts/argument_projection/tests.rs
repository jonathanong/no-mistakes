use super::ArgumentProjection;

#[test]
fn public_constructor_normalizes_unsorted_names_and_union() {
    let mut selected = ArgumentProjection::exact(vec!["z".into(), "a".into(), "z".into()]);
    assert!(selected.matches("a"));
    assert!(selected.matches("z"));
    assert!(!selected.matches("b"));
    assert!(selected.covers(&ArgumentProjection::exact(vec!["z".into()])));
    selected.include(&ArgumentProjection::exact(vec!["b".into()]));
    assert!(selected.covers(&ArgumentProjection::exact(vec!["b".into(), "a".into()])));
    assert!(!selected.covers(&ArgumentProjection::default()));
    selected.include(&ArgumentProjection::default());
    assert!(selected.covers(&ArgumentProjection::default()));
    assert!(selected.matches("arbitrary"));
}
