import { expect } from "@playwright/test";
// no-mistakes-disable-next-line playwright-assertion-timeout-cap -- kept for this fixture
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30000 });
