import type { test, test as typeTest } from "vitest";
import { type it, type it as typeIt } from "vitest";

typeTest.sequential("typed", () => {});
typeIt.sequential("inline type", () => {});
test.sequential("bare type", () => {});
it.sequential("bare inline type", () => {});
