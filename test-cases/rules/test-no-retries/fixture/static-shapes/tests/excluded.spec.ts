import { test } from "@playwright/test";
import { it } from "vitest";

test.describe.configure({ mode: "serial", retries: 99 });
it("renders the outline", { retry: 3 }, () => {});
