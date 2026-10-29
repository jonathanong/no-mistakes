// Only the factory's own importOriginal parameter is the original module.
import { vi } from "vitest";

const other = <T,>(): T => ({}) as T;
vi.mock("../src/wrong-callee.mjs", async (importOriginal) => ({
  ...(await other<typeof import("../src/wrong-callee.mjs")>()),
}));
