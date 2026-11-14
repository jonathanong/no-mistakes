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


review this before i merge

```ts
// test/overdue.test.ts
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { overdueInvoices } from "../src/overdue";
import { seedInvoices } from "./fixtures/invoices";

describe("overdueInvoices", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-15T00:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("returns only unpaid invoices past their due date", () => {
    const result = overdueInvoices(seedInvoices());
    for (const invoice of result) {
      expect(invoice.paidAt).toBeNull();
      expect(invoice.dueDate.getTime()).toBeLessThan(Date.now());
    }
  });

  it("sorts by due date, oldest first", () => {
    const result = overdueInvoices(seedInvoices());
    expect(result.map((i) => i.id)).toEqual(["inv_17", "inv_04", "inv_22"]);
  });

  it("returns an empty list when everything is paid", () => {
    const paid = seedInvoices().filter((i) => i.paidAt !== null);
    expect(overdueInvoices(paid)).toEqual([]);
  });
});
```

seedInvoices() is ~200 rows from a fixture file, mixed paid and unpaid.
