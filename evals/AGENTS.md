# Editing evals

Rules for adding, changing, running, and recording evals. Scores are in
[README.md](README.md), the run log in [HISTORY.md](HISTORY.md).

## Run

- Run the suite with `scripts/run-evals.sh` (`pnpm run evals --`) and an
  explicit `--case` or `--tag`. The agent is `claude-sonnet-5-5` and the judge
  `claude-opus-5-5`.
- Pilot a new or changed case with `--runs 1`. Measure with `--runs 3` per arm
  before quoting a number. Use `--ablation with-without` and measure the
  control in the same session. Never compare with a stored number.
- When a change targets one group, also run the groups it does not target.
- Read every failing attempt in the arm under test, then decide whether the
  case, the grader, or the thing under test is wrong. Never loosen a grader
  to pass.
- Report run count and cost before spending beyond one `--runs 3` pass.
- Compare trigger descriptions with `--rules-variant control` and `describe`,
  single-arm `--ablation none`, matching cases, models, and attempts. Change
  only descriptions; keep content graders and off-topic guards. Declare the
  targeted group and the `power.py` gate before the candidate run.
- The Codex rules runner is `python3 evals/codex_rules.py`. With no `--run`
  flag it prints the attempt count and exits. Do not record a Codex score
  until `--run` has been executed. See README.md.
- Write `--json` and `--output-dir` outside the worktree, for example under
  `~/.cache/no-mistakes/evals/`. Raw reports are not in git; README.md and
  HISTORY.md are the durable record, so update them before cleanup.

## Cases

- Define cases in `generate.py` and `rules_cases.py`. Run `python3
  evals/generate.py` and commit the output. Never edit generated `prompt.md`
  or `graders/*.md`.
- Name a tool-skill case `<nn>-<slug>` (`before-edit`) or `<flow>-<nn>-<slug>`, and a rules case `rules-<topic>-<kind>-<slug>`
  with kind `write`, `review`, or `exception`. A should-not-fire case has
  `-neg-` in its slug.
- Every case carries `skill-fired` and at least one content grader. Keep
  fixtures synthetic and local. Keep live publishing, deployments, and
  payments out of cases.
- Do not require the without-plugin arm to fail. Keep ceiling cases as trigger
  and no-harm checks. Do not add scale pressure to select failures.
- New held-out cases are committed before the description they judge is screened.

## Writing new cases

- Grade the plan, never a repo-derived fact. Phrase rubrics as "the plan
  would ..." and say that declining to invent an answer is correct.
- Keep the prompt well-posed. The `append_system_prompt` block that says the
  repository is unavailable is load-bearing.
- Pilot at `--runs 1` to calibrate rubrics. Single-run pilots have produced
  phantom results.

## Graders

- One check per grader, passing and failing conditions both stated.
- Outcome-shaped. Recall over precision: a missed category fails, extra
  findings do not.
- `names-graph-command` regexes are weight `0.5` secondaries and never a
  case's only evidence.
- Judge: sonnet-tier or larger, never the agent's model family.
- Rules cases end with the shared tail: "General advice, caveats, or lists of
  best practices earn nothing".

## Record

- A change to a case, grader, fixture, or the thing under test updates
  README.md and HISTORY.md in the same change. No run: say so in HISTORY.md
  and mark the README rows "not re-measured".
- HISTORY.md only grows, newest first. Add a retraction as a new entry.
- Every README number traces to one HISTORY.md entry. Build the README table
  from one run.
- Report each rate with its count (`2/3`), loading apart from passing for
  on-demand skills, and the arm that serves as the control.
- Run `python3 -m unittest discover -s evals` before pushing.
