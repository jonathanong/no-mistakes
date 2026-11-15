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


in two or three sentences: what's the difference between a mutex and a semaphore?
