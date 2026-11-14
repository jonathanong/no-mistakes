---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-testing]
append_system_prompt: |
  There is no repository in this environment. Do not read or search
  for files, and do not ask for any to be provided. Put your complete answer
  (the full code, or the full review) in your final message.
---


write vitest tests for readManifest. happy path, a manifest where
one entry has no name, and no manifest.json in the dir.

```ts
import { readFile } from "node:fs/promises";
import { join } from "node:path";

export type ManifestResult = { names: string[]; skipped: number };

export async function readManifest(dir: string): Promise<ManifestResult> {
  let raw: string;
  try {
    raw = await readFile(join(dir, "manifest.json"), "utf8");
  } catch {
    return { names: [], skipped: 0 };
  }
  const entries = JSON.parse(raw).entries as Array<{ name?: string }>;
  const names = entries.flatMap((e) => (e.name ? [e.name] : []));
  return { names, skipped: entries.length - names.length };
}
```
