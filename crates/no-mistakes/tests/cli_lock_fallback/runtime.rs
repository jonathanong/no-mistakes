use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(super) struct ReadOnlyRuntime {
    runtime: PathBuf,
    read_only_directory: PathBuf,
}

impl ReadOnlyRuntime {
    pub(super) fn new(root: &Path) -> Self {
        let runtime = root.join("runtime");
        let read_only_directory = runtime.join("no-mistakes");
        let lock = read_only_directory.join("invocation.lock");
        set_mode(&lock, 0o444);
        set_mode(&read_only_directory, 0o555);

        let runtime = match std::fs::OpenOptions::new().write(true).open(&lock) {
            Err(error) if error.kind() == ErrorKind::PermissionDenied => runtime,
            // Privileged processes can bypass mode bits. A saved regular file
            // still makes the runtime directory unusable for those processes.
            Ok(_) => root.join("repo/message.ts"),
            Err(error) => panic!("checking fixture lock permissions: {error}"),
        };
        Self {
            runtime,
            read_only_directory,
        }
    }

    pub(super) fn path(&self) -> &Path {
        &self.runtime
    }
}

impl Drop for ReadOnlyRuntime {
    fn drop(&mut self) {
        set_mode(&self.read_only_directory, 0o700);
    }
}

pub(super) fn set_mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}
