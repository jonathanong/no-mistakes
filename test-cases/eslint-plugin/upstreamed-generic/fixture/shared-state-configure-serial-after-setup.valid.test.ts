import { test } from "@playwright/test";

let items: string[] = [];

test.describe("suite", () => {
  test.beforeAll(() => {
    items = [];
  });

  test("case", () => {
    items.push("value");
  });

  test.describe.configure({ mode: "serial" });
});
