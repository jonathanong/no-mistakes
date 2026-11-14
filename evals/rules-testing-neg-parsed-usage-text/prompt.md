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


review this before i merge

```ts
// test/cli-usage.test.ts
import { describe, expect, it } from "vitest";
import { renderUsage } from "../src/cli/usage";

describe("renderUsage", () => {
  // scripts/gen-cli-docs.ts parses these exact lines to build the
  // options table in the docs. Changing the format breaks the docs build.
  it("keeps the option line format the docs generator parses", () => {
    const lines = renderUsage().split("\n");
    expect(lines).toContain("  --format <json|text>   output format (default: text)");
    expect(lines).toContain("  --root <dir>           project root (default: cwd)");
  });

  it("lists every registered command", () => {
    const out = renderUsage();
    for (const name of ["check", "dependents", "symbols"]) {
      expect(out).toContain(name);
    }
  });
});
```
