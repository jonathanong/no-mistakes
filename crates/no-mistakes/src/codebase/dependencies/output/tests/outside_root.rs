use super::*;

#[test]
fn paths_keep_absolute_nodes_outside_the_root() {
    let root = p("/root");
    let mut buf = Vec::new();
    write_paths(
        &[
            entry("/other/src/a.mts", 1),
            symbol_entry("/other/src/b.mts", "beta", 1, vec![]),
            queue_job_entry("/other/src/queues.mts", "job", 1),
        ],
        &root,
        &mut buf,
    )
    .unwrap();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("/other/src/a.mts"));
    assert!(s.contains("/other/src/b.mts#beta"));
    assert!(s.contains("/other/src/queues.mts#job"));
}

#[test]
fn human_and_md_render_multiple_roots() {
    let root = p("/root");
    let mut human = Vec::new();
    write_human(
        &["a.mts".to_string(), "b.mts".to_string()],
        &[entry("/root/c.mts", 1)],
        &root,
        &mut human,
    )
    .unwrap();
    assert!(String::from_utf8(human).unwrap().contains("2 files"));
}
