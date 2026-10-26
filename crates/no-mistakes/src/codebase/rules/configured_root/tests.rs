use super::*;
use std::path::Path;

#[test]
fn dot_repository_root_still_rejects_a_parent() {
    assert!(escapes_repository(Path::new("."), Path::new("..")));
    assert!(escapes_repository(
        Path::new("./."),
        Path::new("./../outside")
    ));
    assert!(!escapes_repository(Path::new("."), Path::new("./src")));
    assert!(!escapes_repository(Path::new("."), Path::new(".")));
}
