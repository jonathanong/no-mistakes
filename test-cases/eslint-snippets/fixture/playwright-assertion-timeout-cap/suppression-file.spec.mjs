// no-mistakes-disable-file playwright-assertion-timeout-cap -- kept for this fixture
import { expect } from "@playwright/test";
await expect(page.getByRole("status")).toHaveText("Done", { timeout: 30000 });
