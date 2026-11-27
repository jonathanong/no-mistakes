import { expect } from "@playwright/test";
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30000 });
