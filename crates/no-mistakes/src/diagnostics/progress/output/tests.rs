use super::*;
#[test]
fn blocked_writer_and_full_queue_never_block_emission_or_cleanup() {
    struct BlockedWriter {
        ready: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    }
    impl Write for BlockedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let _ = self.ready.send(());
            self.release.recv().unwrap();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let (ready, started) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    let output = ProgressOutput::start(Ok(BlockedWriter {
        ready,
        release: blocked,
    }));
    let now = std::time::Instant::now();
    output.emit("first".to_string());
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    output.emit("queued".to_string());
    output.emit("dropped".to_string());
    drop(output);
    assert!(now.elapsed() < Duration::from_secs(1));
    release.send(()).unwrap();
    started.recv_timeout(Duration::from_secs(2)).unwrap();
    release.send(()).unwrap();
}

#[test]
fn failed_writer_is_best_effort() {
    let output = ProgressOutput::start(Err::<io::Sink, _>(io::Error::other("closed stderr")));
    output.emit("ignored".to_string());
}
