#!/usr/bin/env python3
"""Saved-fixture integrity and shared-sampler orchestration coverage."""
import copy
import json
from pathlib import Path
import runpy
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
HELPERS = runpy.run_path(str(Path(__file__).with_name("mimalloc-napi-measure.py")))
GLOBALS = HELPERS["main"].__globals__
FIXTURE = ROOT / "fixtures/performance/mimalloc/napi"


class NativeEvidenceTests(unittest.TestCase):
    def test_addon_requires_its_own_hash(self):
        provenance = json.loads((FIXTURE / "addon-provenance.json").read_text())
        self.assertEqual(HELPERS["verify_addon"](FIXTURE, provenance), FIXTURE / "no-mistakes.node")
        changed = copy.deepcopy(provenance)
        changed["no-mistakes.node"]["sha256"] = "0" * 64
        for invalid in (changed, {}):
            with self.assertRaisesRegex(ValueError, "addon hash"):
                HELPERS["verify_addon"](FIXTURE, invalid)

    def test_workload_identity_archive_hash_and_tree_are_required(self):
        record = HELPERS["verify_workload"](FIXTURE, "linux-arm64")
        self.assertEqual(record["commit"], HELPERS["CONSUMER_COMMIT"])
        with self.assertRaisesRegex(ValueError, "provenance mismatch"):
            HELPERS["verify_workload"](FIXTURE, "macos-arm64")
        for field, value, error in (("archive_sha256", "0" * 64, "archive hash"),
                                    ("tree", None, "tree identity")):
            invalid = {**record, field: value}
            with patch.object(Path, "read_text", return_value=json.dumps(invalid)):
                with self.assertRaisesRegex(ValueError, error):
                    HELPERS["verify_workload"](FIXTURE, "linux-arm64")

    def test_sampler_receives_fresh_node_argv_addon_environment_and_metadata(self):
        args = SimpleNamespace(system="/system", mimalloc="/mimalloc", consumer=str(FIXTURE),
                               prepared=str(FIXTURE), output="/output", host_id="host", source="source",
                               jobs="max", pair=1, platform="linux-arm64")
        fields = {key: key for key in ("source", "lock", "lto", "codegenUnits", "rustc", "cargo", "target", "build_flags")}
        calls = []
        shared = {"mimalloc_environment": lambda: {}, "verify_pair": lambda artifacts: None,
                  "verify_artifact": lambda *unused: fields,
                  "measure": lambda *argv, **kwargs: calls.append((argv, kwargs))}

        def output(argv, **unused):
            if argv[-1] == "--version":
                return "v24.0.0\n"
            enabled = "mimalloc" in str(argv[0])
            return json.dumps({"expected_mimalloc": enabled, "is_mimalloc_heap": enabled,
                               "default_rayon_jobs": 4})

        with patch("argparse.ArgumentParser.parse_args", return_value=args), patch.dict(
                GLOBALS, SHARED=shared, verify_addon=lambda directory, provenance: Path(directory) / "no-mistakes.node",
                digest=lambda artifact: HELPERS["digest"](artifact) if Path(artifact).is_file() else str(artifact)), patch("subprocess.check_output", side_effect=output), patch.dict(
                "os.environ", {}, clear=True):
            HELPERS["main"]()
        self.assertEqual(len(calls), 2)
        for position, variant in enumerate(("mimalloc", "system")):
            argv, kwargs = calls[position]
            metadata = argv[4]
            self.assertEqual(argv[0][-2:], [str(FIXTURE), "max"])
            self.assertTrue(argv[0][1].endswith("mimalloc-napi-runner.js"))
            self.assertEqual(argv[5], 600)
            self.assertEqual(metadata["variant"], variant)
            self.assertEqual(metadata["order"], "BA")
            self.assertEqual(metadata["artifact_sha256"], f"/{variant}/no-mistakes.node")
            self.assertEqual(metadata["automatic_rayon_jobs"], 4)
            self.assertEqual(metadata["node_version"], "v24.0.0")
            self.assertEqual(metadata["workload"], "consumer-napi")
            self.assertFalse(metadata["diagnostics"])
            self.assertEqual(kwargs["env"]["NO_MISTAKES_TEST_NAPI_ADDON_PATH"], f"/{variant}/no-mistakes.node")


if __name__ == "__main__":
    unittest.main()
