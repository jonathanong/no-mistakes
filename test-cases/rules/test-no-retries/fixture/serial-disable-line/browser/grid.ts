import { test } from "@playwright/test";

test.describe.configure({ mode: "serial" }); // no-mistakes-disable-line playwright-no-serial
test.describe("sibling", () => {
  test.describe.configure({ mode: "serial" });
});
