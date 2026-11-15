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


Write a Vitest test for this function. Cover the happy path and
the case where the amount is negative.

```ts
export class RefundError extends Error {
  constructor(readonly code: "NEGATIVE_AMOUNT" | "OVER_LIMIT", message: string) {
    super(message);
  }
}

export function refund(amount: number, limit: number): number {
  if (amount < 0) {
    throw new RefundError("NEGATIVE_AMOUNT", `refund amount ${amount} must not be negative`);
  }
  if (amount > limit) {
    throw new RefundError("OVER_LIMIT", `refund amount ${amount} exceeds limit ${limit}`);
  }
  return limit - amount;
}
```
