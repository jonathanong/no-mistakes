use super::*;
use std::time::Duration;
#[test]
fn executor_runs_independently_and_returns_overload_without_waiting() {
    let (started, ready) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let worker = Executor::start(1, move |job| {
        started.send(job).unwrap();
        resume.recv().unwrap();
    });
    worker.submit(1).unwrap();
    assert_eq!(ready.recv_timeout(Duration::from_secs(2)).unwrap(), 1);
    worker.submit(2).unwrap();
    assert_eq!(worker.submit(3), Err(3));
    release.send(()).unwrap();
    assert_eq!(ready.recv_timeout(Duration::from_secs(2)).unwrap(), 2);
    drop(worker);
    release.send(()).unwrap();
}

#[test]
fn disconnected_executor_returns_the_job_without_waiting() {
    let (sender, receiver) = mpsc::sync_channel(1);
    drop(receiver);
    let worker = Executor { sender };
    assert_eq!(worker.submit(7), Err(7));
}
