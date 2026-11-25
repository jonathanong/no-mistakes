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


review this before i merge. CI got 82% line coverage after we
added the label import path. This config diff makes it green.

```diff
--- a/vitest.config.ts
+++ b/vitest.config.ts
@@ -5,8 +5,8 @@ export default defineConfig({
     test: {
       coverage: {
-        reporter: ["text"],
+        reporter: ["text", "json-summary"],
         thresholds: {
-          lines: 90,
+          lines: 80,
           branches: 85,
         },
       },
```
