# Improving the evals

Gaps between `evals/` and the eval-suite conventions, ordered by what each
would let the suite decide. Written 2026-11-26. Nothing here is built. Current
scores are in [`evals/README.md`](../../evals/README.md), the run log in
[`evals/HISTORY.md`](../../evals/HISTORY.md), the editing rules in
[`evals/AGENTS.md`](../../evals/AGENTS.md).

## Priorities

| # | Gap | Why it matters | First step | Cost |
| --- | --- | --- | --- | --- |
| 1 | No priority or target pass rate per case | A row of "2/3" cannot be called pass or fail | Add `priority:` (critical 80%, high 70%, medium 60%, low 50%) to each case in `generate.py` and `rules_cases.py`; emit it in the case files | small |
| 2 | Rates have no 95% interval | `power.py` gives gate margins but tables show bare counts | Add a Wilson interval to `summarize.py` output and to README tables | small |
| 3 | On-demand skills report one layer | `skill-fired` is mixed into the case score, so a skill that loads 1 of 3 times and passes every time it loads looks the same as one that loads 3 of 3 and fails | Report "loaded n/N" and "passed when loaded k/n" separately; hold loading to 80% from six or more attempts; fix loading in the description | medium |
| 4 | Ceiling cases are most of the rules suite | In the 2026-11-26 runs, 6 of 8 `rules-postgresql`, 1 of 5 `rules-queue` and 2 of 6 `rules-testing` cases scored 3/3 in the without-plugin arm. They show the skill fires and does no harm, not that it helps | Mark each ceiling case in the table; add a harder variant per topic, or run a weaker agent model for the ceiling cases | medium |
| 5 | Missing baseline arms and old-rule controls | `--ablation none` runs (the control variant, the Codex runner) have no without-plugin arm in the same session, and no run compares a changed rule's old text with its new text for queue and testing (#4187 and #4623 asked for it) | Always run `with-without` in the session that quotes a delta | small |
| 6 | No early stopping | Every case runs all attempts even when the verdict is fixed | Stop a case when its remaining attempts cannot change pass or fail against the target; record "2 of 2" | medium |
| 7 | No tracking CSV | Scores live in prose across README and HISTORY, so trends and rows without a run are found by reading | One append-only `evals/results.csv` (date, commit, model, judge, case, arm, attempts, loaded, passed, cost) that `summarize.py` appends and the README table is built from | medium |
| 8 | README holds history | Per-grader decompositions, old-model tables, and variant narratives sit in the README | Move to HISTORY.md; keep one table from one run | done in the structure PR, keep it that way |
| 9 | Loading gaps in the rules plugin | Five of 19 measured rules cases loaded 0 or 1 of 3 times on 2026-11-26 (`queue-exception-enqueue-in-tx-receipt` 0/3, `testing-write-flaky-response-wait` 0/3, `testing-exception-raised-coverage-floor` 0/3, `queue-review-entity-only-dedup-id` 1/3, `testing-review-lowered-coverage-floor` 1/3) | Report loading as its own table, then try `describe` variants for queue and testing, six attempts or more per case | $5 per variant |
| 10 | Rows without a current run | The tool-skill table is from 2026-11-21 and the `github-actions` and off-topic guard rows from 2026-11-17 | Re-run `--tag rules-github-actions` and the off-topic guard tag (about $4) and the tool-skill flows (about $45) in one session | $50 |
| 11 | Pilot results are not recorded | A pilot that fails a grader leaves no entry, so the next person repeats it | Add a HISTORY.md entry for any abandoned or auth-failed run | small |

## Grader weaknesses to fix

- `rules-queue-exception-enqueue-in-tx-receipt` scored 0 of 6 on 2026-11-26 and in the 2026-11-17 runs: every answer, with or without the plugin, called the enqueue inside the transaction a defect. Several named a hazard the snippet does not close (a commit failing after `add` succeeds, then a `jobId` collision on redelivery). Both arms failing means the case or its grader is wrong before the rule is: rewrite the snippet so the exception is airtight, then re-pilot.
- `testing-exception-raised-coverage-floor` scored the with arm below baseline with the skill never loaded in 3 of 3. The merge-order warning that both arms raise should not decide the verdict; grade only whether the answer accepts the higher floor.

- `names-graph-command` is a regex that passes when the plan names a command from
  `SKILL.md`. It records use, not a better plan, and carries 57% of the
  should-fire delta in the 2026-11-21 baseline. Pair it with a content grader
  per case, or report it on its own line.
- LLM graders for the new `rules-postgresql` review cases check the finding and a
  repair in one rubric. A review that names the finding but proposes a wrong
  repair, and one that names a correct repair without the finding, both read as
  failing. Split the rubric into a finding check and a repair check so a failing
  attempt says which part failed.
- `skill-fired` on rules cases accepts any `*-rules` skill for off-topic guards
  but only the topic skill for topic cases. Record in the case which one is
  expected, so a wrong skill loaded is visible rather than scored as a miss.
- Exception cases pass when the response leaves the code alone. A response that
  says nothing useful also passes. Add a check that the response answers the
  prompt.

## Method

1. Start every change with a one-attempt pilot to check the graders, then measure
   with three attempts per arm.
2. Measure the control in the same session. Never compare with a number from an
   earlier day.
3. Read every failing attempt before changing a case or grader. Record whether the
   case, the grader, or the skill was wrong.
4. After a run, update HISTORY.md first, then rebuild the README table from the CSV.
5. A change kept on a ceiling score names the specific mistake the rule prevents
   and why a model would make it unprompted.
