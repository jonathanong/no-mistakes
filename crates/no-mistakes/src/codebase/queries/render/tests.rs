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

pub(crate) fn assert_report_writers_surface_io_errors<R: Report>(reports: &[&R]) {
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
    fn exhaust(mut write: impl FnMut(&mut FailAfter) -> io::Result<()>) {
        let mut completed = false;
        for remaining_writes in 0..64 {
            if write(&mut FailAfter { remaining_writes }).is_ok() {
                completed = true;
                break;
            }
        }
        assert!(
            completed,
            "report writers should succeed after enough writes"
        );
    }
    for report in reports {
        exhaust(|writer| report.write_human(writer));
        exhaust(|writer| report.write_paths(writer));
        exhaust(|writer| report.write_md(writer));
    }
}
