import { assertClientOptimizerClosure } from "../../../fixtures/test-timeout-policy/browser-optimizer-receipt.mjs";
import { describe, expect, it } from "vitest";
import { browserFixture } from "../../../fixtures/test-timeout-policy/browser-timeout-child.mjs";
describe("actual Storybook addon generated-test closure contracts", { timeout: 30000 }, () => {
  it.each([false, true])(
    "runs generated story and transitive annotation hooks; stableSetup=%s",
    async (stableSetup) => {
      const result = await browserFixture("", 5000, undefined, { stableSetup });
      expect(result.code, result.output).toBe(0);
      expect(result.browserConsole).toContain("ACTUAL_STORY_PLAY_COMPLETED");
      expect(result.forcedCleanup).toBe(false);
      const routes = result.routing
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      expect(
        routes.some(
          (route) =>
            route.importer?.includes("storybook-registrar-leaf") &&
            route.id === "\0test-timeout-policy:vitest",
        ),
      ).toBe(true);
      expect(
        routes.some(
          (route) =>
            route.importer?.includes("addon-vitest") && route.id === "\0test-timeout-policy:vitest",
        ),
      ).toBe(true);
      const optimizers = assertClientOptimizerClosure(
        result.optimizer,
        [
          "vitest",
          "@vitest/browser",
          "@storybook/addon-vitest",
          "storybook",
          "@storybook/react-vite",
          "@storybook/react",
          "storybook-registrar",
          "storybook-registrar-leaf",
        ],
        "example.stories.mjs",
      );
      for (const optimizer of optimizers) {
        const setup = (
          Array.isArray(optimizer.setupFiles) ? optimizer.setupFiles : [optimizer.setupFiles]
        ).filter((file) => typeof file === "string");
        expect(
          setup.some((file) =>
            file.includes(stableSetup ? "stable.setup.mts" : "setup-file-with-project-annotations"),
          ),
        ).toBe(true);
      }
    },
  );
  it.each([false, true])(
    "rejects aliased transitive annotation hook overcap; stableSetup=%s",
    async (stableSetup) => {
      const result = await browserFixture("", 5000, undefined, { stableSetup, hookTimeout: 30001 });
      expect(result.code).toBe(1);
      expect(result.output).toContain("hook timeout must be positive, finite");
      expect(result.output).not.toContain("ACTUAL_STORY_PLAY_COMPLETED");
      expect(result.browserConsole).not.toContain("ACTUAL_STORY_PLAY_COMPLETED");
      expect(result.forcedCleanup).toBe(false);
    },
  );
});
