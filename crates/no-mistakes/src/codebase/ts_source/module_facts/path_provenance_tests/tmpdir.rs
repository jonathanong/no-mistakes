use super::*;

#[test]
fn node_tmpdir_uses_host_primary_variable_without_changing_direct_env_reads() {
    let read = read("platform-tmpdir.mjs");
    assert_eq!(
        read.origin("readdirSync(tmpdir())").env.as_deref(),
        Some(if cfg!(windows) { "TEMP" } else { "TMPDIR" })
    );
    for name in ["TMPDIR", "TEMP"] {
        assert_eq!(
            read.origin(&format!("readdirSync(process.env.{name})"))
                .env
                .as_deref(),
            Some(name)
        );
    }
}

#[test]
fn dirname_preserves_parent_escapes_and_cancels_only_known_children() {
    let read = read("parent-tmpdir.mjs");
    assert_eq!(
        read.origin("readdirSync(dirname(tmpdir()))").segments,
        [".."]
    );
    assert_eq!(
        read.origin("readdirSync(dirname(dirname(tmpdir())))")
            .segments,
        ["..", ".."]
    );
    assert!(read
        .origin("readdirSync(dirname(join(tmpdir(),")
        .segments
        .is_empty());
    assert_eq!(
        read.origin("readdirSync(dirname(dirname(join(tmpdir(),")
            .segments,
        [".."]
    );
    assert_eq!(
        read.origin("readdirSync(dirname(process.env.TMPDIR))")
            .segments,
        [".."]
    );
    assert_eq!(read.origin("readdirSync(dirname(owned))").segments, [".."]);
    assert_eq!(
        read.origin("readdirSync(dirname(join(owned,").anchor,
        PathAnchor::Temp
    );
}
