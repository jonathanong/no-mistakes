import { test } from "@playwright/test";
import { it } from "vitest";

it("uses fresh state", { retry: 1 }, () => {});
test.describe.serial("shared state", () => {});
// no-mistakes-disable-next-line playwright-no-serial
test.describe.serial("ordered repair", () => {});
