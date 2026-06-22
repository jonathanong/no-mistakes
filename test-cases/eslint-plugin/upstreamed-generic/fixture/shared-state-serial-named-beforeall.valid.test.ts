import { test } from "@playwright/test";

let items: string[] = [];

function resetItems() {
  items = [];
}

test.describe.serial.only("suite", () => {
  test.beforeAll(resetItems);

  test("case", () => {
    items.push("value");
  });
});
