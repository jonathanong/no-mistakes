import { test } from "@playwright/test";

for (const condition of [true, false]) {
  test.slow(condition);
}
