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


what does this regex match? `^\d{3}-\d{4}$`
