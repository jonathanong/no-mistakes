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


add .github/workflows/backend.yml. run `pnpm --filter api test` on
PRs to main, node 24. only run it when something under apps/api/ or
packages/db/ changed, no point burning minutes on frontend PRs.
"backend / test" is a required check on main.
