import copy
import json
import shutil
import tarfile
import tempfile
import unittest
from pathlib import Path

from build_support import remove_root_products, require_dependencies_only, restore_dependencies, safe_environment
from common import digest

FIXTURES = Path(__file__).parent / "fixtures"
EXPECTED = json.loads((FIXTURES / "dependency-manifest.json").read_text())


class DependencyBoundaryTests(unittest.TestCase):
    def prepare(self, temporary, purge=True):
        target = Path(temporary) / "saved-target"
        shutil.copytree(FIXTURES / "dependency-state", target)
        if purge:
            remove_root_products(target)
        archive = Path(temporary) / "dependencies.tar.gz"
        with tarfile.open(archive, "w:gz") as package:
            package.add(target, arcname="target")
        saved = dict(EXPECTED, artifactKind="dependencies-only", archiveSha256=digest(archive))
        return target, archive, saved

    def test_purge_removes_root_crate_only_and_restores_dependencies(self):
        with tempfile.TemporaryDirectory() as temporary:
            target, archive, saved = self.prepare(temporary)
            require_dependencies_only(target)
            self.assertEqual(sorted(str(p.relative_to(target)) for p in target.rglob("*") if p.is_file()), [
                "release/.fingerprint/anyhow-a/marker", "release/build/other-a/output", "release/deps/libanyhow-a.rlib"])
            tree = Path(temporary) / "restored"
            restore_dependencies(saved, EXPECTED, archive, tree)
            require_dependencies_only(tree / "target")
            self.assertEqual((tree / "target/release/deps/libanyhow-a.rlib").read_bytes(),
                             (FIXTURES / "dependency-state/release/deps/libanyhow-a.rlib").read_bytes())

    def test_restore_rejects_stale_provenance_before_extraction(self):
        with tempfile.TemporaryDirectory() as temporary:
            _, archive, saved = self.prepare(temporary)
            for field in (*EXPECTED, "archiveSha256", "artifactKind"):
                with self.subTest(field=field):
                    stale = copy.deepcopy(saved)
                    stale[field] = "stale"
                    tree = Path(temporary) / "never-extracted"
                    with self.assertRaises(ValueError):
                        restore_dependencies(stale, EXPECTED, archive, tree)
                    self.assertFalse(tree.exists())

    def test_restore_rejects_residual_root_products(self):
        with tempfile.TemporaryDirectory() as temporary:
            _, archive, saved = self.prepare(temporary, purge=False)
            with self.assertRaisesRegex(ValueError, "root crate"):
                restore_dependencies(saved, EXPECTED, archive, Path(temporary) / "restored")

    def test_restore_rejects_archive_escape_before_extraction(self):
        for name in ("../escaped", "target/../escaped", "/target/escaped", "source/main.rs"):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                archive = Path(temporary) / "escape.tar.gz"
                with tarfile.open(archive, "w:gz") as package:
                    fixture = FIXTURES / "dependency-manifest.json"
                    member = package.gettarinfo(fixture)
                    member.name = name
                    with fixture.open("rb") as stream:
                        package.addfile(member, stream)
                saved = dict(EXPECTED, artifactKind="dependencies-only", archiveSha256=digest(archive))
                tree = Path(temporary) / "never-extracted"
                with self.assertRaises(ValueError):
                    restore_dependencies(saved, EXPECTED, archive, tree)
                self.assertFalse(tree.exists())

    def test_provenance_excludes_saved_registry_credentials(self):
        environment = json.loads((FIXTURES / "build-environment.json").read_text())
        self.assertEqual(safe_environment(environment["input"]), environment["expected"])

    def test_restore_rejects_saved_symlink_escape(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / "link.tar.gz"
            with tarfile.open(archive, "w:gz") as package:
                package.add(FIXTURES / "dependency-escape", arcname="target")
            saved = dict(EXPECTED, artifactKind="dependencies-only", archiveSha256=digest(archive))
            tree = Path(temporary) / "never-extracted"
            with self.assertRaisesRegex(ValueError, "link points outside target"):
                restore_dependencies(saved, EXPECTED, archive, tree)
            self.assertFalse(tree.exists())


if __name__ == "__main__":
    unittest.main()
