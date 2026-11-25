import { test } from "@playwright/test";

export const config = { retry: 5, retries: 5, workers: 1, fullyParallel: false };

test.describe.configure({ mode: "serial", retries: 99 });
