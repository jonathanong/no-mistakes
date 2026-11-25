import { expect, test } from "vitest";
import { opaqueWorker } from "./worker";

test("exports the worker", () => {
  expect(opaqueWorker).toBeDefined();
});
