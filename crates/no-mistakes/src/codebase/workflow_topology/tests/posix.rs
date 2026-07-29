use super::super::posix_path;

#[test]
fn posix_normalize_resolves_dot_and_dot_dot_segments() {
    assert_eq!(
        posix_path::normalize(".github/workflows/./reusable.yml"),
        ".github/workflows/reusable.yml"
    );
    assert_eq!(
        posix_path::normalize(".github/workflows/../actions/reusable.yml"),
        ".github/actions/reusable.yml"
    );
    assert_eq!(posix_path::normalize("../outside.yml"), "../outside.yml");
    assert_eq!(posix_path::normalize(""), ".");
}

#[test]
fn posix_dirname_of_a_bare_basename_is_dot() {
    assert_eq!(
        posix_path::dirname(".github/workflows/ci.yml"),
        ".github/workflows"
    );
    assert_eq!(posix_path::dirname("ci.yml"), ".");
}
