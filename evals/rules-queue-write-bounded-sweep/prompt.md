---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-queue]
append_system_prompt: |
  There is no repository in this environment. Do not read or search
  for files, and do not ask for any to be provided. Put your complete answer
  (the full code, or the full review) in your final message.
---


nightly job runTrialExpiry(pool): downgrade accounts whose trial
ended to the free plan. node-postgres. accounts has plan,
trial_ends_at. ~4M accounts; a normal day a few thousand trials
end, after a promo it can be 100k+. our scheduler just calls the
function once.
