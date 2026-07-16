use super::deadline::remaining_timeout;
use std::io::Read;
use std::process::{Command, Output, Stdio};
use wait_timeout::ChildExt;

/// Run a child process without allowing it to outlive the active invocation deadline.
pub fn command_output(command: &mut Command) -> std::io::Result<Output> {
    let remaining = remaining_timeout()?;
    let Some(remaining) = remaining else {
        return command.output();
    };

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().expect("child stdout must be piped");
    let stderr = child.stderr.take().expect("child stderr must be piped");
    let stdout_reader = std::thread::spawn(move || read_pipe(stdout));
    let stderr_reader = std::thread::spawn(move || read_pipe(stderr));

    let status = match child.wait_timeout(remaining)? {
        Some(status) => status,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "no-mistakes command deadline elapsed while waiting for a child process",
            ));
        }
    };
    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn read_pipe<R: Read>(mut pipe: R) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(
    reader: std::thread::JoinHandle<std::io::Result<Vec<u8>>>,
) -> std::io::Result<Vec<u8>> {
    reader.join().expect("child output reader must not panic")
}
