# Editing evals

- Keep cases and graders in `generate.py` / `rules_cases.py`; regenerate instead
  of editing generated case folders. Keep fixtures synthetic and local.
- Case selection does not require the without-plugin arm to fail. Keep ceiling
  cases as trigger and content-regression checks. Do not add scale pressure to
  select failures.
- Use Claude Sonnet agents and Opus judges via `scripts/run-evals.sh`, with an
  explicit `--case` or `--tag`. Pilot one case with `--runs 1`; measure groups
  with `--runs 3`. Keep live publishing, deployments, and payments out of cases.
- The Codex rules runner is `python3 evals/codex_rules.py`. With no `--run`
  flag it prints the attempt count and exits. Do not record a Codex score
  until `--run` has been executed. See README.md.
- Compare trigger descriptions with `--rules-variant control` and `describe`,
  single-arm `--ablation none`, matching cases, models, and attempts. Change
  only descriptions; retain content graders and off-topic guards.
- Declare the targeted group and `power.py` gate before the candidate run.
  Inspect every failure and incomplete attempt; never loosen graders to pass.
- Report run count and cost before spending beyond one `--runs 3` pass.
- Record comparable results in README.md and append history in HISTORY.md in
  the same change. Preserve raw reports outside git before cleanup; see README.
