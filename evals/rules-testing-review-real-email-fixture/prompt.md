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
// test/invite.test.ts
import { describe, expect, it } from "vitest";
import { inviteMembers } from "../src/invite";

const members = [
  { name: "Priya", email: "priya.nair@gmail.com" },
  { name: "Tom", email: "tom@acme.com" },
  { name: "Priya (work)", email: "priya.nair@acme.com" },
];

describe("inviteMembers", () => {
  it("sends one invite per distinct address", async () => {
    const sent: string[] = [];
    await inviteMembers(members, async (to) => {
      sent.push(to);
    });
    expect(sent).toEqual([
      "priya.nair@gmail.com",
      "tom@acme.com",
      "priya.nair@acme.com",
    ]);
  });

  it("sends nothing for an empty list", async () => {
    const sent: string[] = [];
    await inviteMembers([], async (to) => {
      sent.push(to);
    });
    expect(sent).toEqual([]);
  });

  it("lowercases addresses before sending", async () => {
    const sent: string[] = [];
    await inviteMembers([{ name: "Sam", email: "Sam@Example.com" }], async (to) => {
      sent.push(to);
    });
    expect(sent).toEqual(["sam@example.com"]);
  });
});
```
