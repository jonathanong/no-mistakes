import * as playwright from "@playwright/test";

playwright.test.describe["configure"]({ mode: inCI ? "parallel" : "serial" });
playwright.test.describe[setting]({ mode: "serial", retries: 99 });
