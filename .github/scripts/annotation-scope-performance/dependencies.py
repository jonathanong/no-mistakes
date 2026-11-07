"""Compile the exact dependency feature graph, excluding every stub product."""
import os
import tarfile
from pathlib import Path

from build_support import cargo_build, inputs, provenance, remove_root_products, source_tree
from common import digest, save

variant, base, head, revision = inputs()
output = Path(os.environ["RUNNER_TEMP"]) / f"annotation-dependencies-{variant}"
output.mkdir()
with source_tree(variant, revision) as tree:
    manifest = provenance(tree, variant, base, head, revision)
    (tree / "crates/no-mistakes/src/lib.rs").write_text("")
    (tree / "crates/no-mistakes/src/main.rs").write_text("fn main() {}\n")
    cargo_build(tree)
    remove_root_products(tree / "target")
    archive = output / "dependencies.tar.gz"
    with tarfile.open(archive, "w:gz", compresslevel=1) as package:
        package.add(tree / "target", arcname="target")
    manifest["archiveSha256"] = digest(archive)
    manifest["artifactKind"] = "dependencies-only"
    save(output / "manifest.json", manifest)
