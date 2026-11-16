---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-github-actions]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


add .github/workflows/lint.yml. on PRs to main, node 24, run
`pnpm lint` and `pnpm typecheck`. read-only, it doesn't push or
comment on anything.
