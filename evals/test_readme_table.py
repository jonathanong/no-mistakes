import contextlib
import io
import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import readme_table
import summarize


def _run(fired, *passed, names=("answer", "names-graph-command")):
    graders = [{"name": "skill-fired", "passed": fired}]
    graders += [
        {"name": n, "passed": p, "weight": 0.5 if n == "names-graph-command" else 1}
        for n, p in zip(names, passed)
    ]
    return {"graders": graders, "error": None}


class ReadmeTableTest(unittest.TestCase):
    def _print(self, kind, cases, tag="t"):
        with tempfile.TemporaryDirectory() as tmp:
            path = pathlib.Path(tmp) / "r.json"
            path.write_text(json.dumps({"partial": False, "suite": {"tagFilters": [tag]}, "cases": cases}))
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                getattr(readme_table, kind)([str(path)])
            return out.getvalue()

    def test_wilson(self):
        self.assertEqual(readme_table.wilson(3, 3), "44-100%")
        self.assertEqual(readme_table.wilson(0, 0), "n/a")

    def test_rules_topic_and_counts(self):
        out = self._print("rules", [{
            "name": "rules-github-actions-write-x",
            "arms": {
                "with": [_run(True, True, names=("a",)), _run(False, True, names=("a",))],
                "without": [_run(False, False, names=("a",))],
            },
        }])
        self.assertIn("| `github-actions` | 1/2 ", out)
        self.assertIn("| `github-actions-write-x` | 1/2 | 2/2 | 1/1 | 0/1 |", out)

    def test_skill_content_delta_leaves_out_command_regex(self):
        out = self._print("skill", [{
            "name": "01-x",
            "arms": {"with": [_run(True, True, True)], "without": [_run(False, True, False)]},
        }])
        self.assertIn("| `t` | 1/1 (21-100%) | +0.33 | +0.00 | — |", out)

    def test_summarize_skip(self):
        runs = [_run(True, True, False)]
        self.assertAlmostEqual(summarize._mean_score(runs), 1 / 1.5)
        self.assertEqual(summarize._mean_score(runs, summarize.USAGE_GRADERS), 1)


if __name__ == "__main__":
    unittest.main()
