import { test } from "@playwright/test";

let items: string[] = [];

test.describe("serial suite", () => {
  test.describe.configure({ mode: "serial" });

  test.beforeAll(() => {
    items = [];
  });

  test("first", () => {
    items.push("first");
  });
});
