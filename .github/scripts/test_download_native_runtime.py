import base64
from contextlib import redirect_stdout, redirect_stderr
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "native_download", Path(__file__).with_name("download-native-runtime.py"))
download = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(download)
FIXTURES = Path(__file__).parent / "fixtures/native-runtime-download"


class NativeRuntimeDownloadTests(unittest.TestCase):
    def setUp(self):
        self.metadata = json.loads((FIXTURES / "metadata.json").read_text())
        self.environment = patch.dict(os.environ, {
            "GITHUB_REPOSITORY": "example/repository", "NATIVE_RUNTIME_RUN_ID": "456",
            "NATIVE_RUNTIME_TARGET": "test-target", "GH_TOKEN": "test-token",
        })
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.previous = Path.cwd()
        os.chdir(self.temporary.name)
        self.addCleanup(os.chdir, self.previous)
        self.calls = []
        self.archive_name = "runtime"
        self.archive_paths = []

    def transport(self, argv, **kwargs):
        self.calls.append((argv, kwargs))
        if "--output" in argv:
            archive = Path(argv[argv.index("--output") + 1])
            self.archive_paths.append(archive)
            with archive.open("wb") as output:
                output.write(base64.b64decode(
                    (FIXTURES / f"{self.archive_name}.zip.base64").read_bytes()))
            return subprocess.CompletedProcess(argv, 0)
        return subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata))

    def run_main(self):
        output, errors = io.StringIO(), io.StringIO()
        with redirect_stdout(output), redirect_stderr(errors):
            code = download.main()
        return code, json.loads(output.getvalue()), errors.getvalue()

    def test_success_extracts_verified_runtime_and_cleans_archive(self):
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            code, report, _ = self.run_main()
        self.assertEqual(code, 0)
        self.assertTrue(report["success"])
        self.assertEqual(Path("target/debug/no-mistakes.exe").read_bytes(),
                         b"native runtime fixture\n")
        self.assertFalse(self.archive_paths[0].exists())
        self.assertEqual(set(report["seconds"]),
                         {"metadata", "download", "digest", "extraction", "total"})
        self.assertEqual(len(self.calls), 2)
        argv, options = self.calls[1]
        for option, value in (("--retry", "2"), ("--retry-max-time", "90"),
                              ("--connect-timeout", "10"), ("--max-time", "25")):
            self.assertEqual(argv[argv.index(option) + 1], value)
        self.assertNotIn("--location-trusted", argv)
        self.assertNotIn("--retry-all-errors", argv)
        self.assertNotIn("--retry-delay", argv)
        self.assertNotIn("test-token", " ".join(argv))
        self.assertIn("test-token", options["input"])
        self.assertGreater(options["timeout"], 0)
        self.assertLessEqual(options["timeout"], 110)

    def test_metadata_has_separate_bounded_timeout(self):
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            result = download.resolve_artifact("example/repository", "456", "test-target",
                                               "test-token", download.Deadline())
        self.assertEqual(result["id"], 123)
        argv, options = self.calls[0]
        self.assertEqual(argv[argv.index("--max-time") + 1], "10")
        self.assertEqual(argv[argv.index("--connect-timeout") + 1], "10")
        self.assertEqual(options["stdout"], subprocess.PIPE)

    def test_permanent_http_failure_is_not_retried_by_python(self):
        with patch.object(download.subprocess, "run", side_effect=
                          subprocess.CalledProcessError(22, ["curl"])) as transport:
            code, report, _ = self.run_main()
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertEqual(transport.call_count, 1)
        self.assertFalse(Path("target").exists())

    def test_digest_mismatch_never_extracts_or_retries(self):
        self.metadata["artifacts"][0]["digest"] = "sha256:" + "0" * 64
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            code, report, _ = self.run_main()
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertEqual(len(self.calls), 2)
        self.assertFalse(Path("target").exists())
        self.assertFalse(self.archive_paths[0].exists())

    def test_traversal_archive_is_rejected_before_extraction(self):
        archive = io.BytesIO(base64.b64decode(
            (FIXTURES / "traversal.zip.base64").read_bytes()))
        with self.assertRaises(download.DownloadError):
            download.extract(archive, Path("target/debug"), download.Deadline())
        self.assertFalse(Path("target").exists())
        self.assertFalse(Path("outside").exists())

    def test_missing_digest_and_ambiguous_artifact_fail_closed(self):
        for artifacts in ([{**self.metadata["artifacts"][0], "digest": None}],
                          self.metadata["artifacts"] * 2):
            with self.subTest(artifacts=artifacts):
                self.metadata["artifacts"] = artifacts
                with patch.object(download.subprocess, "run", side_effect=self.transport):
                    with self.assertRaises(download.DownloadError):
                        download.resolve_artifact("example/repository", "456", "test-target",
                                                   "test-token", download.Deadline())

    def test_catalog_is_bounded_to_ten_pages(self):
        self.metadata = {"artifacts": [{"name": "other"}] * 100}
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            with self.assertRaises(download.DownloadError):
                download.resolve_artifact("example/repository", "456", "test-target",
                                           "test-token", download.Deadline())
        self.assertEqual(len(self.calls), 10)
        self.assertTrue(self.calls[-1][0][-1].endswith("per_page=100&page=10"))

    def test_expired_deadline_does_not_start_transport(self):
        with patch.object(download.time, "monotonic", side_effect=[0, 111]):
            deadline = download.Deadline()
            with patch.object(download.subprocess, "run") as transport:
                with self.assertRaises(TimeoutError):
                    download.metadata("https://api.github.com/example", "test-token", deadline)
                transport.assert_not_called()

    def test_subprocess_deadline_failure_emits_failure_timing(self):
        with patch.object(download.subprocess, "run", side_effect=
                          subprocess.TimeoutExpired(["curl"], 110)):
            code, report, errors = self.run_main()
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertGreaterEqual(report["seconds"]["total"], 0)
        self.assertIn("TimeoutExpired", errors)


if __name__ == "__main__":
    unittest.main()
