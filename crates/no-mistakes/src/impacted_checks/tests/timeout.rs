use super::*;

#[test]
fn deadline_expiration_during_render_publishes_no_output() {
    let report = generate_impacted_checks(&args(&["src/foo.ts"])).unwrap();
    let mut destination = Vec::new();
    let mut checks = 0;

    let error =
        publish_rendered_with_deadline_check(&report, Format::Json, &mut destination, || {
            checks += 1;
            if checks == 2 {
                anyhow::bail!("synthetic impacted-checks timeout");
            }
            Ok(())
        })
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("synthetic impacted-checks timeout"));
    assert!(destination.is_empty());
}

#[test]
fn publication_surfaces_stdout_write_errors() {
    struct FailingWriter;

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("synthetic stdout failure"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let report = generate_impacted_checks(&args(&["src/foo.ts"])).unwrap();
    let error =
        publish_rendered_with_deadline_check(&report, Format::Json, &mut FailingWriter, || Ok(()))
            .unwrap_err();

    assert!(error.to_string().contains("synthetic stdout failure"));
}
