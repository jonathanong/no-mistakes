use super::*;
use crate::invocation::{InvocationError, InvocationErrorKind};

fn deadline() -> anyhow::Error {
    InvocationError::new(InvocationErrorKind::CommandTimeout, "deadline elapsed").into()
}

#[test]
fn failed_spawn_without_a_boundary_remains_bounded_and_releases_the_claim() {
    let output = CliErrorOutput::new(true);
    let report = output.report(&deadline()).unwrap();
    let started = std::time::Instant::now();
    emit_with_spawner(format!("{report}\n"), output.clone(), |task| {
        drop(task);
        Err(io::Error::other("injected thread spawn failure"))
    });
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(output.report_or_wait(&deadline()).is_some());
    output.complete();
    assert!(output.report_or_wait(&deadline()).is_none());
}
