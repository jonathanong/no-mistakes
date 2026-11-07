"""Owned native build trees and provenance shared by dependency/final phases."""
import contextlib
import os
import posixpath
import tarfile
import platform
import shutil
import signal
import subprocess
from pathlib import Path

from common import command, digest, parity, refs

PROFILE = {"lto": "thin", "codegenUnits": 16, "features": "default",
           "cargoBuildJobs": 6, "rustflags": ""}
ROOT_PRODUCTS = tuple(pattern for name in ("no-mistakes", "no_mistakes") for pattern in (
    f"release/{name}*", f"release/lib{name}*", f"release/deps/{name}*",
    f"release/deps/lib{name}*", f"release/.fingerprint/{name}-*",
    f"release/build/{name}-*", f"release/incremental/{name}-*"))


def inputs():
    variant = os.environ["VARIANT"]
    if variant not in ("base", "candidate"):
        raise ValueError("unknown variant")
    base, head = refs(os.environ["BASE_SHA"], os.environ["HEAD_SHA"])
    return variant, base, head, base if variant == "base" else head


@contextlib.contextmanager
def source_tree(variant, revision):
    parent = Path("/tmp") / f"nm-annotation-{variant}"
    parent.mkdir()  # Existing paths are never adopted or deleted.
    tree = parent / "source"
    added = False
    try:
        subprocess.run(["git", "worktree", "add", "--detach", str(tree), revision], check=True)
        added = True
        yield tree
    finally:
        shutil.rmtree(tree / "target", ignore_errors=True)
        if added:
            subprocess.run(["git", "worktree", "remove", "--force", str(tree)], check=True)
        shutil.rmtree(parent)


def provenance(tree, variant, base, head, revision):
    return {
        "variant": variant, "source": revision, "base": base, "head": head,
        "compiler": command("rustc", "+stable", "-Vv"), "profile": PROFILE,
        "cargo": command("cargo", "+stable", "-Vv"),
        "sourceTreeHash": command("git", "rev-parse", "HEAD^{tree}", cwd=tree),
        "linker": command("xcrun", "--find", "clang"),
        "linkerVersion": command("xcrun", "clang", "--version"),
        "sdk": command("xcrun", "--show-sdk-path"),
        "sdkVersion": command("xcrun", "--show-sdk-version"),
        "buildEnvironment": safe_environment(build_environment(tree)),
        "system": platform.system(), "machine": platform.machine(),
        "os": platform.platform(), "runnerImage": os.environ.get("ImageVersion"),
        "sourceTree": str(tree), "cargoHome": os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
        "sourceManifests": {name: digest(tree / name) for name in
                            ("Cargo.lock", "Cargo.toml", "crates/no-mistakes/Cargo.toml")},
    }


def safe_environment(environment):
    keys = {"CC", "CXX", "CFLAGS", "CXXFLAGS", "LDFLAGS", "RUSTFLAGS",
            "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
            "RUSTUP_TOOLCHAIN", "CARGO_HOME", "CARGO_BUILD_TARGET", "CARGO_BUILD_JOBS",
            "CARGO_INCREMENTAL", "CARGO_TARGET_DIR", "SDKROOT", "MACOSX_DEPLOYMENT_TARGET",
            "NO_MISTAKES_BUILD_NAPI"}
    return {name: value for name, value in environment.items()
            if name in keys or name.startswith("CARGO_PROFILE_")}


def build_environment(tree):
    return dict(os.environ, CARGO_BUILD_JOBS="6", CARGO_TARGET_DIR=str(tree / "target"),
                CARGO_PROFILE_RELEASE_LTO="thin", CARGO_PROFILE_RELEASE_CODEGEN_UNITS="16",
                CARGO_INCREMENTAL="0", RUSTFLAGS="", CARGO_ENCODED_RUSTFLAGS="")


def cargo_build(tree):
    env = build_environment(tree)
    build = subprocess.Popen(["flock", "/tmp/nm-heavy.lock", "cargo", "+stable", "build", "--release",
                              "--locked", "-p", "no-mistakes", "--bin", "no-mistakes"],
                             cwd=tree, env=env, start_new_session=True)
    try:
        build.wait(timeout=1440)
        if build.returncode:
            raise subprocess.CalledProcessError(build.returncode, build.args)
    finally:
        if build.poll() is None:
            os.killpg(build.pid, signal.SIGKILL)
            build.wait(timeout=10)


def root_products(target):
    return [path for pattern in ROOT_PRODUCTS for path in target.glob(pattern)]


def remove_root_products(target):
    for path in root_products(target):
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    require_dependencies_only(target)


def require_dependencies_only(target):
    if root_products(target):
        raise ValueError("dependency artifact contains root crate build products")


def restore_dependencies(saved, expected, archive, tree):
    for field, value in expected.items():
        parity(value, saved.get(field), f"dependency provenance {field}")
    parity("dependencies-only", saved.get("artifactKind"), "dependency artifact kind")
    parity(saved.get("archiveSha256"), digest(archive), "dependency archive SHA256")
    with tarfile.open(archive, "r:gz") as package:
        for member in package.getmembers():
            if (not member.name.startswith("target/") and member.name != "target") or ".." in Path(member.name).parts:
                raise ValueError("dependency archive contains a path outside target")
            if member.issym() or member.islnk():
                link = posixpath.normpath(posixpath.join(posixpath.dirname(member.name), member.linkname)
                                         if member.issym() else member.linkname)
                if not link.startswith("target/"):
                    raise ValueError("dependency archive link points outside target")
        package.extractall(tree, filter="data")
    require_dependencies_only(tree / "target")
