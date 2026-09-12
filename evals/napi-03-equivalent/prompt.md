---
model: claude-opus-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../skills/no-mistakes]
tags: [napi]
append_system_prompt: |
  You are in plan mode. You must not make any edits or other side-effecting
  changes.

  The repository is not available in this environment. Do not attempt to read or
  search it, and do not ask for it to be provided. Produce the concrete plan you
  would execute against the real checkout, at the level of detail someone could
  follow without you.
---
Context: the `no-mistakes` repository itself. A Rust workspace under
`crates/no-mistakes` implements the CLI. `packages/no-mistakes` wraps it as an
N-API addon with a JS facade and hand-maintained `.d.ts` declarations. Docs live
under `docs/` (`docs/cli/*`, `docs/node-api.md`, `docs/rules/*`).

what's the programmatic equivalent of `no-mistakes impacted-checks`?
