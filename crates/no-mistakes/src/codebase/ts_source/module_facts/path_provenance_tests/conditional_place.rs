use super::{read, write, WriteScope};

#[test]
fn conditional_place_marks_an_if_or_ternary_and_the_enclosing_scope() {
    let read = read("env-conditional-place.mjs");
    let provenance = read.provenance();
    assert!(matches!(
        write(provenance, "T").scope,
        WriteScope::Conditional(place)
            if place.branch && place.program && place.hook.is_none() && place.function.is_none()
    ));
    assert!(matches!(
        write(provenance, "U").scope,
        WriteScope::Conditional(place) if place.branch && place.program
    ));
    assert!(matches!(
        write(provenance, "W").scope,
        WriteScope::Conditional(place)
            if place.branch && place.function.is_some() && place.hook.is_none() && !place.program
    ));
    assert!(matches!(
        write(provenance, "H").scope,
        WriteScope::Conditional(place)
            if place.branch && place.hook.is_some() && place.function.is_none() && !place.program
    ));
    assert!(matches!(
        write(provenance, "X").scope,
        WriteScope::Conditional(place) if !place.branch && place.program
    ));
    assert!(matches!(
        write(provenance, "Y").scope,
        WriteScope::Conditional(place) if !place.branch && place.program
    ));
    assert!(matches!(
        write(provenance, "Z").scope,
        WriteScope::Conditional(place) if !place.branch && place.program
    ));
}
