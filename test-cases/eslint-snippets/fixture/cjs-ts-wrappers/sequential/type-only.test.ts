import type { test as typeTest } from "vitest";
import { type it as typeIt } from "vitest";

typeTest.sequential("typed", () => {});
typeIt.sequential("inline type", () => {});
