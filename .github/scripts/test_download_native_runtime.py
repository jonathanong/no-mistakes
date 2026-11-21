import base64
from contextlib import redirect_stdout, redirect_stderr
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
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
            artifact_id = argv[-1].split("/")[-2]
            name = artifact_id if artifact_id in ("124", "125") else self.archive_name
            with archive.open("wb") as output:
                output.write(base64.b64decode(
                    (FIXTURES / f"{name}.zip.base64").read_bytes()))
            return subprocess.CompletedProcess(argv, 0)
        return subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata))

    def run_main(self, names=()):
        output, errors = io.StringIO(), io.StringIO()
        with redirect_stdout(output), redirect_stderr(errors):
            code = download.main(names)
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
        for option, value in (("--connect-timeout", "10"),
                              ("--continue-at", "-"),
                              ("--speed-limit", "1024"), ("--speed-time", "30")):
            self.assertEqual(argv[argv.index(option) + 1], value)
        self.assertNotIn("--max-time", argv)
        self.assertNotIn("--retry", argv)
        self.assertIn("--speed-limit", argv)
        self.assertIn("--speed-time", argv)
        self.assertGreaterEqual(download.OVERALL_SECONDS, 177)
        self.assertNotIn("--location-trusted", argv)
        self.assertNotIn("--retry-all-errors", argv)
        self.assertNotIn("--retry-delay", argv)
        self.assertNotIn("test-token", " ".join(argv))
        self.assertIn("test-token", options["input"])
        self.assertGreater(options["timeout"], 0)
        self.assertLessEqual(options["timeout"], download.OVERALL_SECONDS)

    def test_stall_retries_keep_partial_bytes(self):
        downloads = {"count": 0}

        def transport(argv, **kwargs):
            self.calls.append((argv, kwargs))
            if "--output" not in argv:
                return subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata))
            archive = Path(argv[argv.index("--output") + 1])
            downloads["count"] += 1
            if downloads["count"] == 1:
                archive.write_bytes(b"partial")
                raise subprocess.CalledProcessError(28, argv)
            self.assertTrue(archive.is_file())
            self.assertEqual(archive.read_bytes(), b"partial")
            self.assertIn("--continue-at", argv)
            with archive.open("wb") as output:
                output.write(base64.b64decode((FIXTURES / "runtime.zip.base64").read_bytes()))
            return subprocess.CompletedProcess(argv, 0)

        with patch.object(download.subprocess, "run", side_effect=transport):
            code, report, _ = self.run_main()
        self.assertEqual(code, 0)
        self.assertTrue(report["success"])
        self.assertEqual(downloads["count"], 2)

    def test_range_rejection_discards_partial_and_downloads_fresh(self):
        downloads = {"count": 0}

        def transport(argv, **kwargs):
            self.calls.append((argv, kwargs))
            if "--output" not in argv:
                return subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata))
            archive = Path(argv[argv.index("--output") + 1])
            downloads["count"] += 1
            if downloads["count"] == 1:
                archive.write_bytes(b"partial")
                raise subprocess.CalledProcessError(33, argv)
            self.assertFalse(archive.exists())
            self.assertNotIn("--continue-at", argv)
            self.assertNotIn("--max-time", argv)
            with archive.open("wb") as output:
                output.write(base64.b64decode((FIXTURES / "runtime.zip.base64").read_bytes()))
            return subprocess.CompletedProcess(argv, 0)

        with patch.object(download.subprocess, "run", side_effect=transport):
            code, report, _ = self.run_main()
        self.assertEqual(code, 0)
        self.assertTrue(report["success"])
        self.assertEqual(Path("target/debug/no-mistakes.exe").read_bytes(),
                         b"native runtime fixture\n")

    def test_http_failure_during_download_is_not_retried(self):
        def transport(argv, **kwargs):
            self.calls.append((argv, kwargs))
            if "--output" in argv:
                raise subprocess.CalledProcessError(22, argv)
            return subprocess.CompletedProcess(argv, 0, json.dumps(self.metadata))

        with patch.object(download.subprocess, "run", side_effect=transport):
            code, report, _ = self.run_main()
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertEqual(sum("--output" in argv for argv, _ in self.calls), 1)

    def test_metadata_has_separate_bounded_timeout(self):
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            result = download.resolve_artifact("example/repository", "456", "native-runtime-test-target",
                                               "test-token", download.Deadline())
        self.assertEqual(result["id"], 123)
        argv, options = self.calls[0]
        self.assertEqual(argv[argv.index("--max-time") + 1], "10")
        self.assertEqual(argv[argv.index("--connect-timeout") + 1], "10")
        self.assertEqual(options["stdout"], subprocess.PIPE)

    def test_transports_tolerate_offline_windows_revocation_servers(self):
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            code, _, _ = self.run_main()
        self.assertEqual(code, 0)
        self.assertGreaterEqual(len(self.calls), 2)
        for argv, _ in self.calls:
            self.assertIn("--ssl-revoke-best-effort", argv)

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
                        download.resolve_artifact("example/repository", "456", "native-runtime-test-target",
                                                   "test-token", download.Deadline())

    def test_catalog_is_bounded_to_ten_pages(self):
        self.metadata = {"artifacts": [{"name": "other"}] * 100}
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            with self.assertRaises(download.DownloadError):
                download.resolve_artifact("example/repository", "456", "native-runtime-test-target",
                                           "test-token", download.Deadline())
        self.assertEqual(len(self.calls), 10)
        self.assertTrue(self.calls[-1][0][-1].endswith("per_page=100&page=10"))

    def test_expired_deadline_does_not_start_transport(self):
        with patch.object(download.time, "monotonic", side_effect=[0, download.OVERALL_SECONDS + 1]):
            deadline = download.Deadline()
            with patch.object(download.subprocess, "run") as transport:
                with self.assertRaises(TimeoutError):
                    download.metadata("https://api.github.com/example", "test-token", deadline)
                transport.assert_not_called()

    def test_subprocess_deadline_failure_emits_failure_timing(self):
        with patch.object(download.subprocess, "run", side_effect=
                          subprocess.TimeoutExpired(["curl"], download.OVERALL_SECONDS)):
            code, report, errors = self.run_main()
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertGreaterEqual(report["seconds"]["total"], 0)
        self.assertIn("TimeoutExpired", errors)

    def test_linux_components_complete_through_real_process_without_network(self):
        fixtures = Path("transport")
        shutil.copytree(FIXTURES, fixtures)
        (fixtures / "curl").chmod(0o755)
        env = {**os.environ, "PATH": str(fixtures.resolve()) + os.pathsep + os.environ["PATH"],
               "NATIVE_DOWNLOAD_FIXTURES": str(fixtures.resolve())}
        result = subprocess.run([sys.executable, str(Path(download.__file__).resolve()),
                                 "linux-runtime-cli", "linux-runtime-addon"],
                                env=env, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["success"])
        self.assertEqual([row["name"] for row in report["artifacts"]],
                         ["linux-runtime-cli", "linux-runtime-addon"])
        self.assertEqual(Path("target/debug/no-mistakes").read_text(),
                         "linux-runtime-cli fixture\\n")
        self.assertEqual(Path("target/debug/no-mistakes.node").read_text(),
                         "linux-runtime-addon fixture\\n")
        for stage in ("metadata", "download", "digest", "extraction"):
            self.assertAlmostEqual(report["seconds"][stage],
                                   sum(row["seconds"][stage] for row in report["artifacts"]))

    def test_cli_only_consumer_completes_without_extracting_addon(self):
        fixtures = Path("transport")
        shutil.copytree(FIXTURES, fixtures)
        (fixtures / "curl").chmod(0o755)
        env = {**os.environ, "PATH": str(fixtures.resolve()) + os.pathsep + os.environ["PATH"],
               "NATIVE_DOWNLOAD_FIXTURES": str(fixtures.resolve())}
        result = subprocess.run([sys.executable, str(Path(download.__file__).resolve()),
                                 "linux-runtime-cli"],
                                env=env, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertTrue(report["success"])
        self.assertEqual([row["name"] for row in report["artifacts"]], ["linux-runtime-cli"])
        self.assertEqual(Path("target/debug/no-mistakes").read_text(),
                         "linux-runtime-cli fixture\\n")
        self.assertFalse(Path("target/debug/no-mistakes.node").exists())

    def test_explicit_names_reject_duplicates_and_invalid_names_before_transport(self):
        for names in (["linux-runtime-cli"] * 2, [""], ["../runtime"], ["runtime/name"]):
            with self.subTest(names=names), patch.object(download.subprocess, "run") as transport:
                code, report, _ = self.run_main(names)
                self.assertEqual(code, 1)
                self.assertFalse(report["success"])
                transport.assert_not_called()

    def test_linux_catalog_missing_duplicate_expired_and_bad_digest_fail(self):
        saved = json.loads((FIXTURES / "linux-metadata.json").read_text())
        first, second = saved["artifacts"]
        for artifacts in ([first], [first, second, second],
                          [first, {**second, "expired": True}],
                          [first, {**second, "digest": None}]):
            with self.subTest(artifacts=artifacts):
                self.metadata = {"artifacts": artifacts}
                with patch.object(download.subprocess, "run", side_effect=self.transport):
                    with self.assertRaises(download.DownloadError):
                        download.resolve_artifact("example/repository", "456",
                                                  "linux-runtime-addon", "test-token", download.Deadline())

    def test_second_linux_component_digest_mismatch_never_extracts_addon(self):
        self.metadata = json.loads((FIXTURES / "linux-metadata.json").read_text())
        self.metadata["artifacts"][1]["digest"] = "sha256:" + "0" * 64
        with patch.object(download.subprocess, "run", side_effect=self.transport):
            code, report, errors = self.run_main(["linux-runtime-cli", "linux-runtime-addon"])
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertEqual([row["name"] for row in report["artifacts"]], ["linux-runtime-cli"])
        self.assertTrue(Path("target/debug/no-mistakes").is_file())
        self.assertFalse(Path("target/debug/no-mistakes.node").exists())
        self.assertEqual(len(self.calls), 4)
        self.assertTrue(all(not archive.exists() for archive in self.archive_paths))
        self.assertIn("SHA256 mismatch", errors)

    def test_linux_components_share_one_overall_deadline(self):
        self.metadata = json.loads((FIXTURES / "linux-metadata.json").read_text())
        now = [0]
        original = self.transport
        def transport(argv, **kwargs):
            result = original(argv, **kwargs)
            if "--output" in argv:
                now[0] = (download.OVERALL_SECONDS - 1
                          if argv[-1].endswith("/124/zip") else download.OVERALL_SECONDS + 2)
            elif len(self.calls) == 3:
                now[0] = download.OVERALL_SECONDS + 1
            return result
        with patch.object(download.time, "monotonic", side_effect=lambda: now[0]), \
                patch.object(download.subprocess, "run", side_effect=transport):
            code, report, errors = self.run_main(["linux-runtime-cli", "linux-runtime-addon"])
        self.assertEqual(code, 1)
        self.assertFalse(report["success"])
        self.assertEqual(len(report["artifacts"]), 1)
        self.assertEqual(len(self.calls), 3)
        self.assertTrue(Path("target/debug/no-mistakes").is_file())
        self.assertFalse(Path("target/debug/no-mistakes.node").exists())
        self.assertIn(f"{download.OVERALL_SECONDS} seconds", errors)


if __name__ == "__main__":
    unittest.main()
