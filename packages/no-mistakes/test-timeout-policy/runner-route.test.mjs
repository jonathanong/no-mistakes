import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe.skipIf(process.platform !== "linux")(
  "real SDK config runner module routing (unqualified source proposals)",
  () => {
    it.each([false, true])(
      "worker-loads the final own-field runner; standalone=%s",
      async (standalone) => {
        const result = await fixture(
          fixtureSource("runner-route.test-31.txt"),
          undefined,
          "",
          undefined,
          {},
          { standalone },
        );
        expect(result.code, result.output).toBe(0);
        expect(result.marker).toBe("continued");
        expect(result.extensionMarker).toBe("own field initialized");
        expect(result.lifecycle).toBe("final constructor initialized");
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.each([false, true])(
      "worker-loads the native default and retains importFile; standalone=%s",
      async (standalone) => {
        const result = await fixture(
          fixtureSource("runner-route.test-32.txt"),
          undefined,
          "",
          undefined,
          {},
          { nativeDefault: true, standalone },
        );
        expect(result.code, result.output).toBe(0);
        expect(result.marker).toBe("continued");
        expect(result.extensionMarker).toBeUndefined();
        expect(result.lifecycle).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.each([false, true])(
      "rejects final own-field config overcap; standalone=%s",
      async (standalone) => {
        const result = await fixture(
          fixtureSource("runner-route.test-33.txt"),
          undefined,
          "",
          undefined,
          {},
          {
            standalone,
            configTimeout: 30001,
          },
        );
        expect(result.code).toBe(1);
        expect(result.output).toContain("testTimeout must be positive, finite");
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it("rejects the native-default resolved config before body", async () => {
      const result = await fixture(
        fixtureSource("runner-route.test-34.txt"),
        fixtureTemplate("runner-route.test-template-1.txt"),
        "",
        undefined,
        {},
        { nativeDefault: true },
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("testTimeout must be positive, finite");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    });
    it("rejects post-extension task overcap on the worker-loaded final runner", async () => {
      const result = await fixture(
        fixtureSource("runner-route.test-35.txt"),
        undefined,
        "",
        undefined,
        {},
        { taskTimeout: 30001 },
      );
      expect(result.code).toBe(1);
      expect(result.output).toContain("Vitest resolved test timeout");
      expect(result.marker).toBeUndefined();
      expect(result.forcedCleanup).toBe(false);
    });
    it.each([false, true])(
      "rejects hook overcap before body; nativeDefault=%s",
      async (nativeDefault) => {
        const result = await fixture(
          fixtureSource("runner-route.test-36.txt"),
          undefined,
          "",
          undefined,
          {},
          { nativeDefault },
        );
        expect(result.code).toBe(1);
        expect(result.output).toContain("hook timeout must be positive, finite");
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.each(["mergedOverride", "cliOverride"])(
      "rejects %s of the policy-selected runner",
      async (kind) => {
        const result = await fixture(
          fixtureSource("runner-route.test-37.txt"),
          undefined,
          "",
          undefined,
          {},
          { [kind]: true },
        );
        expect(result.code, result.output).toBe(1);
        expect(result.output).toContain(
          kind === "cliOverride" ? "Unknown option `--runner`" : "Final runner policy was replaced",
        );
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
  },
);
