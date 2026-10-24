#[test]
fn child_process_group_receives_forwarded_termination_signal() {
    use std::io::BufRead;

    // Serialize with deadline tests; open piped stdin holds the shell without sleeping.
    let _serial = deadline_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut command = Command::new("sh");
    command
        .args([
            "-c",
            "trap 'exit 42' TERM; printf '%s\\n' ready; IFS= read -r input",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped());
    super::child::configure_process_group(&mut command);
    let mut child = command.spawn().unwrap();
    let process_tree = super::child::ProcessTree::attach(&child);
    let stdout = child.stdout.take().unwrap();
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        let result = std::io::BufReader::new(stdout)
            .read_line(&mut line)
            .map(|_| line);
        let _ = ready_sender.send(result);
    });
    let ready = ready_receiver.recv_timeout(Duration::from_secs(10));
    let is_ready = matches!(&ready, Ok(Ok(line)) if line == "ready\n");
    if !is_ready {
        let terminate = process_tree.terminate(&mut child);
        let wait = child.wait_timeout(Duration::from_secs(1));
        if !matches!(wait, Ok(Some(_))) {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = reader.join();
        panic!(
            "child did not report signal-handler readiness: {ready:?}; termination: {terminate:?}"
        );
    }
    reader.join().unwrap();

    signals::forward_signal(child.id() as i32, signal_hook::consts::SIGTERM);
    let Some(status) = child.wait_timeout(Duration::from_secs(1)).unwrap() else {
        let terminate = process_tree.terminate(&mut child);
        let wait = child.wait_timeout(Duration::from_secs(1));
        if !matches!(wait, Ok(Some(_))) {
            let _ = child.kill();
            let _ = child.wait();
        }
        panic!(
            "forwarded signal did not terminate the child process group; termination: {terminate:?}"
        );
    };
    drop(process_tree);
    assert_eq!(status.code(), Some(42));
}
