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


This Playwright test sometimes times out waiting for the save
response, especially when the local app responds quickly.
My proposed fix is below. Can you give me the final test code?

```diff
import { expect, test } from "@playwright/test";

+test.describe.configure({ retries: 2 });
+
test("saves the selected label", async ({ page }) => {
  await page.goto("/labels");
  await page.getByRole("button", { name: "Save label" }).click();
  const response = await page.waitForResponse((response) =>
    new URL(response.url()).pathname === "/api/labels" &&
    response.request().method() === "POST");
  expect(response.ok()).toBe(true);
});
```
