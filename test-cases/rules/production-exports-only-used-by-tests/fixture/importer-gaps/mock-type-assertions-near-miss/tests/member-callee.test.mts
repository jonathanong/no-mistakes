// A member call is not the importOriginal parameter.
import { vi } from "vitest";

const helpers = { load: <T,>(): T => ({}) as T };
vi.mock("../src/member-callee.mjs", async () => ({
  ...(await helpers.load<typeof import("../src/member-callee.mjs")>()),
}));
