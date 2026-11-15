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


publish.yml: on a v* tag push, publish the package to npm. only
publish if our CI workflow (ci.yml, it has jobs lint and test)
passed for that commit. trusted publishing, node 24.
