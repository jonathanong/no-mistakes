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
