import { test as base } from "@playwright/test";
import { wrap } from "missing-playwright-wrapper";

const test = wrap(base);
test.describe.configure({ mode: "serial", retries: 2 });
