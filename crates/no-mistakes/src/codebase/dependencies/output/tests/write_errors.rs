use super::*;
use std::io::{self, Write};

struct FailAfter {
    remaining_writes: usize,
}

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining_writes == 0 {
            return Err(io::Error::other("synthetic write failure"));
        }
        self.remaining_writes -= 1;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn exhaust_write(mut write: impl FnMut(&mut FailAfter) -> Result<()>) {
    let mut completed = false;
    for remaining_writes in 0..4096 {
        if write(&mut FailAfter { remaining_writes }).is_ok() {
            assert!(remaining_writes > 0);
            completed = true;
            break;
        }
    }
    assert!(completed, "writer should succeed after enough writes");
}

fn mixed_entries() -> Vec<NodeEntry> {
    vec![
        entry("/root/src/a.mts", 1),
        symbol_entry("/root/src/b.mts", "beta", 1, vec![]),
        queue_job_entry("/root/src/queues.mts", "job", 1),
        workflow_job_entry("/root/.github/workflows/ci.yml", "test", 1),
        workflow_step_entry("/root/.github/workflows/ci.yml", "test", 0, 1),
        trpc_procedure_entry("/root/src/router.mts", "getUser", 1),
        module_entry("lodash", 1, vec![EdgeKind::Import]),
    ]
}

#[test]
fn output_writers_surface_io_errors() {
    let root = p("/root");
    let entries = mixed_entries();
    let roots = ["src/a.mts".to_string(), "src/b.mts".to_string()];
    exhaust_write(|writer| write_json(&roots, &entries, &root, writer));
    exhaust_write(|writer| write_json(&["src/a.mts".to_string()], &[], &root, writer));
    exhaust_write(|writer| write_json_with_diagnostics(&roots, &entries, &root, &[], &[], writer));
    exhaust_write(|writer| write_paths(&entries, &root, writer));
    exhaust_write(|writer| write_human(&roots, &entries, &root, writer));
    exhaust_write(|writer| write_human(&["src/a.mts".to_string()], &[], &root, writer));
    exhaust_write(|writer| write_md(&roots, &entries, &root, writer));
    exhaust_write(|writer| write_md(&["src/a.mts".to_string()], &[], &root, writer));
    exhaust_write(|writer| write_yml(&roots, &entries, &root, writer));
    exhaust_write(|writer| write_yml_with_diagnostics(&roots, &entries, &root, &[], &[], writer));
}
