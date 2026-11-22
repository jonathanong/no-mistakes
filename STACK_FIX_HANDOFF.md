# Fragment-escape stack handoff

Work paused at the user's request. This WIP branch combines the new lower-layer
and upper-layer fixes; it is not a validated replacement for either PR head.
The PR branches have not been pushed by this follow-up lane.

Implemented:

- Record a structured gap when partial analysis has no canonical graph and a
  bare module specifier cannot be proven; preserve relative-import findings.
- Report missing prepared facts for supported TS/JS graph files. Non-source
  files such as tsconfig.json remain outside this rule.
- Read literal created-require calls by their recorded call offsets, including
  bindings named load and member calls such as again.resolve.
- Add saved fixtures and Rust regression assertions for both omissions; update
  the rule documentation. Existing snapshots have not been regenerated.

Remaining:

- Run focused fragment-rule unit tests, architecture tests, and
  suite_cli_postgres_part1 fragment-rule snapshot tests; deliberately regenerate
  and inspect the absence-gap and literal-positions snapshots.
- Check formatting and physical line limits after splitting the changes back
  onto their lowest owning PR layers, then restack onto current main.
- Build and measure each final head against v0.109.110, observer off. The release
  baseline build was interrupted; no new timing results exist.
- Correct both PR bodies to identify the exact measured source revisions, push
  with leases to the PR branches only after resuming authorization, and shepherd
  both PRs to READY. Never merge or rerun CI to investigate flakes.

Private benchmark scripts and prior evidence are retained under
`~/.cache/no-mistakes/3802-review` and `~/.cache/no-mistakes/3802-takeover`.
The new measure3.py harness requires distinct verified binary hashes and source
provenance for the release baseline and both final heads.
