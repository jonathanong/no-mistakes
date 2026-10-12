import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { assertClientOptimizerClosure } from "../../../fixtures/test-timeout-policy/browser-optimizer-receipt.mjs";
import { describe, expect, it } from "vitest";
import { browserFixture } from "../../../fixtures/test-timeout-policy/browser-timeout-child.mjs";
describe.skipIf(process.platform !== "linux")(
  "actual Browser Mode routing contracts",
  { timeout: 30000 },
  () => {
    it("routes the SDK tester constructor and preserves browser body/context execution", async () => {
      const result = await browserFixture(fixtureSource("browser-route.test-23.txt"));
      expect(result.code, result.output).toBe(0);
      expect(result.forcedCleanup).toBe(false);
      const routes = result.routing
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      expect(
        routes.some(
          (route) =>
            route.importer?.includes("tester-") && route.id === "\0test-timeout-policy:vitest",
        ),
      ).toBe(true);
      assertClientOptimizerClosure(
        result.optimizer,
        ["vitest", "@vitest/browser"],
        "case.test.mts",
      );
    });
    it("rejects browser constructor config before test body", async () => {
      const result = await browserFixture(fixtureSource("browser-route.test-24.txt"), 30001);
      expect(result.code).toBe(1);
      expect(result.output).toContain("testTimeout must be positive, finite");
      expect(result.output).not.toContain("BODY_EXECUTED");
      expect(result.forcedCleanup).toBe(false);
    });
    it("rejects aliased runtime config before body continuation", async () => {
      const result = await browserFixture(fixtureSource("browser-route.test-25.txt"));
      expect(result.code).toBe(1);
      expect(result.output).toContain("testTimeout must be positive, finite");
      expect(result.output).not.toContain("BODY_CONTINUED");
      expect(result.forcedCleanup).toBe(false);
    });
    it("routes a package registrar before native hook timeout capture", async () => {
      const result = await browserFixture(
        fixtureSource("browser-route.test-26.txt"),
        250,
        fixtureTemplate("browser-route.test-template-1.txt"),
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite");
      expect(result.output).not.toContain("BODY_EXECUTED");
      expect(result.forcedCleanup).toBe(false);
    });
  },
);
