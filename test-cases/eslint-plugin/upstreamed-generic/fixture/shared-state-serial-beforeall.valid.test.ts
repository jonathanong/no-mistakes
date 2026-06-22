import { test as base } from "@playwright/test";

let browserName = "";
const serialTest = base.extend({});

serialTest.describe.serial("suite", () => {
  serialTest.beforeAll(() => {
    browserName = "chromium";
  });

  serialTest("case", () => {
    expect(browserName).toBe("chromium");
  });
});
