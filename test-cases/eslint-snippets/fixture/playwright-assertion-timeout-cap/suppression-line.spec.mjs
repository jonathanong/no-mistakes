import { expect } from "@playwright/test";
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30000 }); // no-mistakes-disable-line playwright-assertion-timeout-cap -- kept for this fixture
