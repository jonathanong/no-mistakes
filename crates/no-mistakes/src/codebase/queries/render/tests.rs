use super::*;

#[derive(Serialize)]
struct FailingReport;

impl Report for FailingReport {
    fn write_human(&self, output: &mut dyn Write) -> io::Result<()> {
        output.write_all(b"partial")?;
        Err(io::Error::other("synthetic render failure"))
    }

    fn write_paths(&self, _output: &mut dyn Write) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn render_failure_does_not_publish_a_partial_report() {
    let mut destination = Vec::new();

    let error = render(&FailingReport, Format::Human, &mut destination).unwrap_err();

    assert!(error.to_string().contains("writing human output"));
    assert!(destination.is_empty());
}

#[test]
fn deadline_after_buffering_does_not_publish_a_report() {
    let mut destination = Vec::new();
    let mut checks = 0;

    let error = render_with_deadline_check(&FailingReport, Format::Paths, &mut destination, || {
        checks += 1;
        if checks == 2 {
            anyhow::bail!("synthetic timeout");
        }
        Ok(())
    })
    .unwrap_err();

    assert!(error.to_string().contains("synthetic timeout"));
    assert!(destination.is_empty());
}
