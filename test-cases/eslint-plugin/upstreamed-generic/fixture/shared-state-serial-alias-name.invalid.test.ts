import { test } from "@playwright/test";

const serial = test.extend({});
let items: string[] = [];

serial.describe("suite", () => {
  serial.beforeAll(() => {
    items = [];
  });

  serial("case", () => {
    items.push("value");
  });
});
