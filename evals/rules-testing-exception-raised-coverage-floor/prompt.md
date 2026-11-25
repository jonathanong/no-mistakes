---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-testing]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


review this before i merge. We added tests for the label
importer in a separate PR. The same CI command now reports 94%
line coverage with the same include and exclude patterns.
This diff locks in part of that gain.

```diff
--- a/vitest.config.ts
+++ b/vitest.config.ts
@@ -6,7 +6,7 @@ export default defineConfig({
     test: {
       coverage: {
         thresholds: {
-          lines: 85,
+          lines: 92,
           branches: 85,
         },
       },
```
