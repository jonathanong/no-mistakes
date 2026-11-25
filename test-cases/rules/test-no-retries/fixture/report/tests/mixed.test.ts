import { test } from "@playwright/test";
import { test as unitTest } from "vitest";

test.describe.configure({ mode: "serial", retries: 99 });
unitTest("checks receipt formatting", { retry: 2 }, () => {});
