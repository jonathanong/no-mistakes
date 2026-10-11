import { test, beforeEach, vi } from "vitest";
import { defineConfig } from "vitest/config";
// Direct sinks must reject runtime functions without an opaque use tainting them first.
test("low callable", { timeout: () => 1 }, () => {});
test("high callable", { timeout: () => 60_000 }, () => {});
const low = () => 1;
test("callable alias", { timeout: low }, () => {});
beforeEach(() => {}, { timeout: () => 1 });
vi.setConfig({ testTimeout: () => 1, hookTimeout: () => 60_000 });
defineConfig({ test: { testTimeout: () => 1, hookTimeout: () => 60_000 } });
test("numeric control", { timeout: 1 }, () => {});
test("high numeric control", { timeout: 60_000 }, () => {});
defineConfig(() => ({ test: { testTimeout: 1 } }));
