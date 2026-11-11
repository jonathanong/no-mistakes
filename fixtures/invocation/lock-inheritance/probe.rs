use std::ffi::c_void;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

unsafe extern "C" {
    fn read(fd: i32, buffer: *mut c_void, count: usize) -> isize;
    fn write(fd: i32, buffer: *const c_void, count: usize) -> isize;
    fn close(fd: i32) -> i32;
    fn getpid() -> i32;
}

fn state(path: &Path) -> &'static str {
    let probe = File::open(path).unwrap();
    match probe.try_lock_shared() {
        Ok(()) => {
            probe.unlock().unwrap();
            "free"
        }
        Err(TryLockError::WouldBlock) => "held",
        Err(error) => panic!("status probe failed: {error}"),
    }
}

fn main() {
    let path = std::env::temp_dir().join(format!("nm-lock-probe-{}", std::process::id()));
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    held.try_lock().unwrap();
    let held_fd = held.as_raw_fd();
    let (mut ready, child_ready) = UnixStream::pair().unwrap();
    let (mut release, child_release) = UnixStream::pair().unwrap();
    let parent_ready_fd = ready.as_raw_fd();
    let parent_release_fd = release.as_raw_fd();
    ready
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let spawning = std::thread::spawn(move || {
        let ready_fd = child_ready.as_raw_fd();
        let release_fd = child_release.as_raw_fd();
        let mut command = Command::new("/bin/true");
        unsafe {
            command.pre_exec(move || {
                close(parent_ready_fd);
                close(parent_release_fd);
                let pid = getpid().to_ne_bytes();
                let mut byte = [0];
                if write(ready_fd, pid.as_ptr().cast(), pid.len()) != pid.len() as isize
                    || read(release_fd, byte.as_mut_ptr().cast(), 1) != 1
                {
                    return Err(std::io::Error::from_raw_os_error(5));
                }
                Ok(())
            });
        }
        let mut child = command.spawn().unwrap();
        drop((child_ready, child_release));
        child.wait().unwrap()
    });
    let mut pid = [0; 4];
    ready.read_exact(&mut pid).unwrap();
    let pid = i32::from_ne_bytes(pid);
    let before = state(&path);
    drop(held);
    let after_drop = state(&path);
    if cfg!(target_os = "linux") {
        let descriptor = std::fs::read_link(format!("/proc/{pid}/fd/{held_fd}")).unwrap();
        assert_eq!(descriptor, path);
        println!(
            "child {pid} inherited fd {held_fd}: {}",
            descriptor.display()
        );
    }
    release.write_all(b"x").unwrap();
    assert!(spawning.join().unwrap().success());
    let after_exec = state(&path);
    std::fs::remove_file(path).unwrap();
    println!(
        "before drop: {before}; after drop: {after_drop}; after child exec/exit: {after_exec}"
    );
    assert_eq!((before, after_drop, after_exec), ("held", "held", "free"));
}
