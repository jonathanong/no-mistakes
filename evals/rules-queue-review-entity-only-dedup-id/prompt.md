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


review this before i merge

```ts
// src/search/enqueue.ts
import { searchQueue } from "../queues";
import { logger } from "../logger";

const retention = {
  removeOnComplete: { age: 86_400 },
  removeOnFail: { age: 604_800 },
};

export async function requestReindex(docId: string, reason: string) {
  await searchQueue.add(
    "reindex-document",
    { docId, reason },
    { ...retention, jobId: `reindex-${docId}` },
  );
}

export function recordView(docId: string, userId: string) {
  // best effort analytics; losing a view is fine
  void searchQueue
    .add("track-view", { docId, userId })
    .catch((err) => logger.warn({ err, docId }, "track-view enqueue failed"));
}

// src/docs/events.ts
export async function onDocumentUpdated(event: { id: string; docId: string }) {
  await requestReindex(event.docId, "updated");
}
```

the worker for reindex-document is idempotent.
