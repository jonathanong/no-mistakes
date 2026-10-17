import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe.skipIf(process.platform !== "linux")(
  "candidate public TestRunner constructor composition (Node probes)",
  () => {
    it("preserves cached identity, most-derived prototype and custom context lifecycle", async () => {
      const result = await fixture(
        fixtureTemplate("public-runner.test-template-6.txt"),
        undefined,
        fixtureTemplate("public-runner.test-template-5.txt"),
      );
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it("guards the final public SDK subclass with an own-field extension and private receiver", async () => {
      const result = await fixture(
        fixtureTemplate("public-runner.test-template-4.txt"),
        undefined,
        fixtureTemplate("public-runner.test-template-3.txt"),
      );
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it.each(["30001", "0", "NaN", "Infinity"])(
      "rejects raw configured deadline %s during public construction",
      async (timeout) => {
        const result = await fixture(
          fixtureTemplate("public-runner.test-template-2.txt"),
          undefined,
          fixtureTemplate("public-runner.test-template-1.txt", [timeout]),
        );
        expect(result.code).toBe(1);
        expect(result.output).toContain(
          "testTimeout must be positive, finite and at most 30000 ms",
        );
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
  },
);
