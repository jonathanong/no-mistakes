"""Restore validated compiler dependencies, then build and freeze actual source."""
import json
import os
import subprocess
import shutil
from pathlib import Path

from build_support import cargo_build, inputs, provenance, restore_dependencies, source_tree
from common import command, digest, parity, save

variant, base, head, revision = inputs()
output = Path(os.environ["RUNNER_TEMP"]) / f"annotation-{variant}"
output.mkdir()
with source_tree(variant, revision) as tree:
    manifest = provenance(tree, variant, base, head, revision)
    dependencies = Path(os.environ["RUNNER_TEMP"]) / f"annotation-dependencies-{variant}"
    saved = json.loads((dependencies / "manifest.json").read_text())
    archive = dependencies / "dependencies.tar.gz"
    restore_dependencies(saved, manifest, archive, tree)
    parity(revision, command("git", "rev-parse", "HEAD", cwd=tree), "final source HEAD")
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--"], cwd=tree, check=True)
    cargo_build(tree)
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--"], cwd=tree, check=True)
    binary = output / "no-mistakes"
    shutil.copy2(tree / "target/release/no-mistakes", binary)
    manifest.update({
        "binarySha256": digest(binary), "version": command(str(binary), "--version"),
        "cpu": command("sysctl", "-n", "machdep.cpu.brand_string"),
        "availableCpus": os.cpu_count(), "dependencyArchiveSha256": saved["archiveSha256"],
    })
    save(output / "manifest.json", manifest)
