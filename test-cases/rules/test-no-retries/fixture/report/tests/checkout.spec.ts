import { test as browserTest } from "@playwright/test";

browserTest.describe.configure({ retries: 3, mode: "serial" });
browserTest.describe.configure({ retries: 0, mode: "parallel" });
browserTest.describe.serial("checkout", () => {});
// no-mistakes-disable-next-line playwright-no-serial
browserTest.describe.serial("ordered migration", () => {});
