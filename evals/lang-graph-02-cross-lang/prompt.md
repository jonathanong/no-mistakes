---
model: claude-opus-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../skills/no-mistakes]
tags: [lang-graph]
append_system_prompt: |
  You are in plan mode. You must not make any edits or other side-effecting
  changes.

  The repository is not available in this environment. Do not attempt to read or
  search it, and do not ask for it to be provided. Produce the concrete plan you
  would execute against the real checkout, at the level of detail someone could
  follow without you.
---
Context: a polyglot monorepo. TypeScript services under `services/*`, Python
workers under `workers/*`, and Go tooling under `cmd/*`. The Python and Go
dependency graphs are configured explicitly in `.no-mistakes.yml`; nothing is
inferred by convention. Python tests are pytest, Go tests are standard `go test`.

the go cli and the python worker both read the same config schema. if i change it, what do i need to touch?
