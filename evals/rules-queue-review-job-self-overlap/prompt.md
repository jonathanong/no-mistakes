---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-queue]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


Review this BullMQ change before it ships. All worker replicas
register the same queue, and cron and the admin console both
add a fresh job for the same catalog. The worker reads a
snapshot and replaces the catalog file at a shared path.
A late write from an older snapshot can undo newer data.

```diff
-export const refreshCatalog = (job: Job<CatalogJob>) =>
-  withCatalogLease(job.data.catalogId, () =>
-    rebuildCatalog(job.data.catalogId));
+export const refreshCatalog = (job: Job<CatalogJob>) =>
+  rebuildCatalog(job.data.catalogId);

 new Worker("catalog-refresh", refreshCatalog, { concurrency: 1 });
 // Nightly cron enqueues each catalog; sooner via admin refresh.
```

The author says a worker processes one job at a time and the
job usually finishes in a minute. Is the removal safe?
