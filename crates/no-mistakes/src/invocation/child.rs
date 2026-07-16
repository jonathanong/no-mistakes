use super::deadline::remaining_timeout;
use std::io::Read;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use wait_timeout::ChildExt;

pub(super) mod process_tree;
#[cfg(not(unix))]
use process_tree::configure_process_group;
#[cfg(unix)]
pub(super) use process_tree::configure_process_group;
pub(super) use process_tree::ProcessTree;

const CLEANUP_TIMEOUT: Duration = Duration::from_millis(100);
type PipeReader = Receiver<std::io::Result<Vec<u8>>>;

/// Run a child process without allowing it to outlive the active invocation deadline.
pub fn command_output(command: &mut Command) -> std::io::Result<Output> {
    let remaining = remaining_timeout()?;
    let Some(remaining) = remaining else {
        return command.output();
    };

    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_process_group(command);
    let mut child = command.spawn()?;
    #[cfg(unix)]
    let process_tree =
        ProcessTree::attach(&child, super::deadline::parent_signal_forwarding_enabled());
    #[cfg(not(unix))]
    let process_tree =
        match ProcessTree::attach(&child, super::deadline::parent_signal_forwarding_enabled()) {
            Ok(process_tree) => process_tree,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait_timeout(CLEANUP_TIMEOUT);
                return Err(error);
            }
        };
    let stdout = child.stdout.take().expect("child stdout must be piped");
    let stderr = child.stderr.take().expect("child stderr must be piped");
    let stdout_reader = spawn_reader(stdout);
    let stderr_reader = spawn_reader(stderr);

    let status = match child.wait_timeout(remaining) {
        Ok(Some(status)) => status,
        Ok(None) => {
            return cleanup_wait_error(
                child,
                &process_tree,
                stdout_reader,
                stderr_reader,
                std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "no-mistakes command deadline elapsed while waiting for a child process",
                ),
            );
        }
        Err(err) => {
            return cleanup_wait_error(child, &process_tree, stdout_reader, stderr_reader, err)
        }
    };
    let stdout = receive_or_terminate(&stdout_reader, &mut child, &process_tree)?;
    let stderr = receive_or_terminate(&stderr_reader, &mut child, &process_tree)?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

pub(super) fn cleanup_wait_error(
    mut child: std::process::Child,
    process_tree: &ProcessTree,
    stdout_reader: PipeReader,
    stderr_reader: PipeReader,
    error: std::io::Error,
) -> std::io::Result<Output> {
    let cleanup_error = process_tree.terminate(&mut child).err();
    let _ = child.wait_timeout(CLEANUP_TIMEOUT);
    let _ = stdout_reader.recv_timeout(CLEANUP_TIMEOUT);
    let _ = stderr_reader.recv_timeout(CLEANUP_TIMEOUT);
    cleanup_result(error, cleanup_error)
}

pub(super) fn cleanup_result(
    error: std::io::Error,
    cleanup_error: Option<std::io::Error>,
) -> std::io::Result<Output> {
    match cleanup_error {
        Some(cleanup_error) => Err(std::io::Error::new(
            error.kind(),
            format!("{error}; terminating the child process tree failed: {cleanup_error}"),
        )),
        None => Err(error),
    }
}

pub(super) fn spawn_reader<R: Read + Send + 'static>(pipe: R) -> PipeReader {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read_pipe(pipe));
    });
    receiver
}

pub(super) fn receive_reader(reader: &PipeReader) -> std::io::Result<Vec<u8>> {
    match remaining_timeout()? {
        Some(remaining) => match reader.recv_timeout(remaining) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "no-mistakes command deadline elapsed while reading child output",
            )),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "child output reader stopped before returning output",
            )),
        },
        None => match reader.recv() {
            Ok(result) => result,
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "child output reader stopped before returning output",
            )),
        },
    }
}

fn receive_or_terminate(
    reader: &PipeReader,
    child: &mut std::process::Child,
    process_tree: &ProcessTree,
) -> std::io::Result<Vec<u8>> {
    match receive_reader(reader) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            let _ = process_tree.terminate(child);
            Err(error)
        }
    }
}

pub(super) fn read_pipe<R: Read>(mut pipe: R) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)?;
    Ok(bytes)
}
