import { describe, it as unitTest } from "vitest";

describe("checkout", { retry: 2 }, () => {
  unitTest("shows a receipt", { retry: 1 }, () => {});
  unitTest("reports payment failure", { retry: 0 }, () => {});
  unitTest("falls back to cash", { retry: process.env.CI ? 2 : 0 }, () => {});
});
