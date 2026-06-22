import { test } from "@playwright/test";

const client = {
  configure() {},
};
let shared: string[] = [];

test.describe("suite", () => {
  client.configure({ mode: "serial" });

  test.beforeAll(() => {
    shared = [];
  });

  test("uses shared state", () => {
    shared.push("value");
  });
});
