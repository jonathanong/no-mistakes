---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-offtopic]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


draft a two-sentence release note for v2.3 of our notes app. it adds dark mode and fixes a crash when importing very large markdown files.
