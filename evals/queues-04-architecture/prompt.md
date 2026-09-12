---
model: claude-opus-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../skills/no-mistakes]
tags: [queues]
append_system_prompt: |
  You are in plan mode. You must not make any edits or other side-effecting
  changes.

  The repository is not available in this environment. Do not attempt to read or
  search it, and do not ask for it to be provided. Produce the concrete plan you
  would execute against the real checkout, at the level of detail someone could
  follow without you.
---
Context: the `auto-harness` monorepo. pnpm workspaces under `actions/*`, `modules/*`
(client, shared, ui) and `services/*` (api, cdk, host-daemon, host-pane, web).
`modules/shared` is consumed by the other packages as `@auto-harness/shared` through a
single barrel entrypoint. `services/web` is a Next.js App Router app under `src/app/`.
Unit tests are Vitest, colocated beside sources as `*.test.ts(x)`. Playwright specs live
in `e2e/`.

explain the architecture of the outbox queues — producers, consumers, and what connects them
