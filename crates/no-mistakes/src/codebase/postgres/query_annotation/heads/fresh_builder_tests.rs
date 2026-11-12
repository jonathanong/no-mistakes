use super::super::HeadProof;
use super::support::{assert_markers, fixture_with_effects_from};

fn assert_case(name: &str, marker: &str) {
    let (source, _, heads) = fixture_with_effects_from("append-only-shapes", name);
    assert_eq!(assert_markers(&source, &heads), 8);
    let start = source.find(marker).unwrap() as u32;
    assert_eq!(
        heads.calls.get(&(start, start + marker.len() as u32)),
        Some(&HeadProof::Literal),
    );
}

#[test]
fn mutable_fresh_builders_require_every_assignment_and_use_to_preserve_the_literal_head() {
    assert_case("g-mutable.mts", "read(statement)");
}

#[test]
fn conditional_fresh_builders_require_every_literal_arm_and_a_closed_append_only_lifetime() {
    assert_case("h-conditional.mts", "read(q)");
}

#[test]
fn conditional_fresh_builders_accept_only_source_proven_local_effects() {
    let (source, _, heads) = fixture_with_effects_from("append-only-shapes", "h-local-effects.mts");
    assert_eq!(assert_markers(&source, &heads), 3);
}
