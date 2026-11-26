use super::read;

#[test]
fn captured_member_wrappers_allow_only_unchanged_forwarding_and_readonly_inspection() {
    let read = read("captured-wrappers.mjs");
    let mut forwarded: Vec<_> = read
        .provenance()
        .pass_through
        .iter()
        .map(|(offset, name)| {
            (
                read.source[..*offset as usize].lines().count(),
                name.as_str(),
            )
        })
        .collect();
    forwarded.sort();
    assert_eq!(
        forwarded,
        [
            (4, "readdir"),
            (6, "readdirSync"),
            (31, "readFile"),
            (34, "readdir"),
            (38, "readdir")
        ]
    );
}
