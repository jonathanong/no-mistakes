"""Build and freeze one exact source revision; never share target directories."""
import os
import platform
import signal
import shutil
import subprocess
import tempfile
from pathlib import Path

from common import command, digest, refs, save

variant = os.environ["VARIANT"]
if variant not in ("base", "candidate"):
    raise ValueError("unknown variant")
base, head = refs(os.environ["BASE_SHA"], os.environ["HEAD_SHA"])
revision = base if variant == "base" else head
output = Path(os.environ["RUNNER_TEMP"]) / f"annotation-{variant}"
output.mkdir()
with tempfile.TemporaryDirectory(prefix="annotation-build-") as temporary:
    tree = Path(temporary) / "source"
    subprocess.run(["git", "worktree", "add", "--detach", str(tree), revision], check=True)
    build = None
    try:
        env = dict(os.environ, CARGO_BUILD_JOBS="6", CARGO_TARGET_DIR=str(tree / "target"),
                   CARGO_PROFILE_RELEASE_LTO="thin", CARGO_PROFILE_RELEASE_CODEGEN_UNITS="16",
                   RUSTFLAGS="")
        build = subprocess.Popen(["flock", "/tmp/nm-heavy.lock", "cargo", "+stable", "build", "--release",
                                  "--locked", "-p", "no-mistakes", "--bin", "no-mistakes"],
                                 cwd=tree, env=env, start_new_session=True)
        try:
            build.wait(timeout=1440)
        except subprocess.TimeoutExpired:
            os.killpg(build.pid, signal.SIGKILL)
            build.wait(timeout=10)
            raise
        if build.returncode:
            raise subprocess.CalledProcessError(build.returncode, build.args)
        binary = output / "no-mistakes"
        shutil.copy2(tree / "target/release/no-mistakes", binary)
        save(output / "manifest.json", {
            "variant": variant, "source": revision, "base": base, "head": head,
            "binarySha256": digest(binary), "compiler": command("rustc", "+stable", "-Vv"),
            "version": command(str(binary), "--version"), "system": platform.system(),
            "machine": platform.machine(), "os": platform.platform(),
            "cpu": command("sysctl", "-n", "machdep.cpu.brand_string"),
            "availableCpus": os.cpu_count(), "runnerImage": os.environ.get("ImageVersion"),
            "profile": {"lto": "thin", "codegenUnits": 16, "features": "default",
                        "cargoBuildJobs": 6, "rustflags": ""},
        })
    finally:
        if build is not None and build.poll() is None:
            os.killpg(build.pid, signal.SIGKILL)
            build.wait(timeout=10)
        shutil.rmtree(tree / "target", ignore_errors=True)
        subprocess.run(["git", "worktree", "remove", "--force", str(tree)], check=True)
