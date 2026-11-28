import type { default as check } from "@playwright/test";
import { type test as typedTest } from "vitest";

check.skip("type only declaration", () => {});
typedTest.skip("type only specifier", () => {});
