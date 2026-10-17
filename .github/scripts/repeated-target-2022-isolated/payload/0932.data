import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe.skipIf(process.platform !== "linux")(
  "public collector and data factory SDK contracts",
  () => {
    it.each(["beforeAll", "afterAll"])(
      fixtureTemplate("review-public-surfaces.test-template-4.txt"),
      async (hook) => {
        const result = await fixture(
          fixtureTemplate("review-public-surfaces.test-template-3.txt", [hook]),
        );
        expect(result.code, result.output).toBe(1);
        expect(result.output).toContain("hook timeout must be positive, finite");
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.each(["describe.each", "suite.for"])(
      "retains hook-shaped callable table rows through %s",
      async (registration) => {
        const result = await fixture(
          fixtureTemplate("review-public-surfaces.test-template-2.txt", [registration]),
        );
        expect(result.code, result.output).toBe(0);
        expect(result.marker).toBe("continued");
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it("preserves private static APIs on actual public SDK runner subclasses", async () => {
      const result = await fixture(
        fixtureSource("review-public-surfaces.test-18.txt"),
        undefined,
        fixtureTemplate("review-public-surfaces.test-template-1.txt"),
      );
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it("retains valid returned-test hooks and rejects overcap test registration", async () => {
      const result = await fixture(fixtureSource("review-public-surfaces.test-19.txt"));
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
  },
);
