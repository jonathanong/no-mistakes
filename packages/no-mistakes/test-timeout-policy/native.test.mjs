import { fixtureTemplate } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe.skipIf(process.platform !== "linux")(
  "native Vitest registration and runtime deadlines",
  () => {
    it.concurrent.each([
      fixtureTemplate("native.test-template-19.txt"),
      fixtureTemplate("native.test-template-18.txt"),
      fixtureTemplate("native.test-template-17.txt"),
      fixtureTemplate("native.test-extra-template-6.txt"),
      fixtureTemplate("native.test-extra-template-5.txt"),
      fixtureTemplate("native.test-extra-template-4.txt"),
      fixtureTemplate("native.test-extra-template-3.txt"),
      fixtureTemplate("native.test-template-16.txt"),
      fixtureTemplate("native.test-template-15.txt"),
      fixtureTemplate("native.test-template-14.txt"),
      fixtureTemplate("native.test-template-13.txt"),
      fixtureTemplate("native.test-template-12.txt"),
    ])("rejects during collection before the body: %s", async (source) => {
      const result = await fixture(source);
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toContain("must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
    });
    it.concurrent.each(["30001", "0", "-1", "NaN", "Infinity"])(
      "rejects opaque runtime configuration through a setter alias: %s",
      async (timeout) => {
        const result = await fixture(fixtureTemplate("native.test-template-11.txt", [timeout]));
        expect(result.signal).toBeNull();
        expect(result.code).toBe(1);
        expect(result.output).toContain("must be positive, finite and at most 30000 ms");
        expect(result.marker).toBeUndefined();
      },
    );
    it.concurrent.each(["0", "-1", "NaN", "Infinity", "30001"])(
      "rejects a resolved standalone project timeout before collection: %s",
      async (timeout) => {
        const result = await fixture(
          fixtureSource("native.test-6.txt"),
          fixtureTemplate("native.test-extra-template-2.txt", [timeout]),
        );
        expect(result.signal).toBeNull();
        expect(result.code).toBe(1);
        expect(result.marker).toBeUndefined();
      },
    );
    it.concurrent("guards context completion hooks before their timers are created", async () => {
      const result = await fixture(fixtureSource("native.test-7.txt"));
      expect(result.code).toBe(1);
      expect(result.signal).toBeNull();
      expect(result.output).toContain("must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
    });
    it.concurrent("rejects raw native config writes through the runner configuration", async () => {
      const result = await fixture(fixtureSource("native.test-8.txt"));
      expect(result.code).toBe(1);
      expect(result.signal).toBeNull();
      expect(result.output).toContain("must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
    });
    it.concurrent.each([
      fixtureTemplate("native.test-template-10.txt"),
      fixtureTemplate("native.test-template-9.txt"),
      fixtureTemplate("native.test-template-8.txt"),
    ])(
      "validates resolved tasks before timer construction despite raw imports: %s",
      async (source) => {
        const result = await fixture(source);
        expect(result.signal).toBeNull();
        expect(result.code).toBe(1);
        expect(result.output).toContain(
          "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
        );
        expect(result.marker).toBeUndefined();
      },
    );
    it.concurrent.each([
      fixtureTemplate("native.test-template-7.txt"),
      fixtureTemplate("native.test-template-6.txt"),
    ])("rejects a delegated extension invalidating the original task: %s", async (mutation) => {
      const result = await fixture(
        fixtureSource("native.test-9.txt"),
        undefined,
        fixtureTemplate("native.test-extra-template-1.txt", [mutation]),
      );
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toContain(
        "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
      );
      expect(result.extensionMarker).toBe("called");
      expect(result.marker).toBeUndefined();
    });
    it.concurrent("rejects an invalid raw deadline before an extension can mask it", async () => {
      const result = await fixture(
        fixtureSource("native.test-10.txt"),
        undefined,
        fixtureTemplate("native.test-template-5.txt"),
      );
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toContain(
        "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
      );
      expect(result.extensionMarker).toBeUndefined();
      expect(result.marker).toBeUndefined();
    });
    it.concurrent.each([false, true])(
      "routes package and path aliases with preserveSymlinks=%s",
      async (preserveSymlinks) => {
        const results = await Promise.all(
          ["deadline-helper", "registration-alias"].map((specifier) =>
            fixture(fixtureTemplate("native.test-template-4.txt", [specifier]), undefined, "", {
              preserveSymlinks,
              source: fixtureTemplate("native.test-template-3.txt"),
            }),
          ),
        );
        for (const result of results) {
          expect(result.signal).toBeNull();
          expect(result.code).toBe(1);
          expect(result.output).toContain(
            "hook timeout must be positive, finite and at most 30000 ms",
          );
          expect(result.marker).toBeUndefined();
        }
      },
    );
    it.concurrent("preserves around-hook execution with the composed context guard", async () => {
      const result = await fixture(fixtureSource("native.test-11.txt"));
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it.concurrent.each([false, true])(
      "preserves legal package helpers with symlinks=%s",
      async (preserveSymlinks) => {
        const result = await fixture(fixtureSource("native.test-12.txt"), undefined, "", {
          preserveSymlinks,
          source: fixtureTemplate("native.test-template-2.txt"),
        });
        expect(result.code, result.output).toBe(0);
        expect(result.marker).toBe("continued");
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.concurrent.each(["aroundAll", "aroundEach"])(
      "retains native %s short-hang termination",
      async (hook) => {
        const result = await fixture(fixtureTemplate("native.test-template-1.txt", [hook]));
        expect(result.code).toBe(1);
        expect(result.signal).toBeNull();
        expect(result.output).toMatch(/timed out/i);
        expect(result.marker).toBeUndefined();
        expect(result.forcedCleanup).toBe(false);
      },
    );
    it.concurrent("retains native short-hang termination", async () => {
      const result = await fixture(fixtureSource("native.test-13.txt"));
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toMatch(/timed out/i);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it.concurrent("accepts the inclusive configured ceiling and aliases", async () => {
      const result = await fixture(fixtureSource("native.test-14.txt"));
      expect(result.signal).toBeNull();
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it.concurrent("forwards exactly the validated options snapshot", async () => {
      const result = await fixture(fixtureSource("native.test-15.txt"));
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toMatch(/timed out/i);
      expect(result.marker).toBe("continued");
    });
    it.concurrent("preserves an undefined optional deadline and typed fixture data", async () => {
      const result = await fixture(fixtureSource("native.test-16.txt"));
      expect(result.signal).toBeNull();
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    });
    it.concurrent(
      "uses separate outer containment for a blocked event loop",
      { timeout: 10000 },
      async () => {
        const result = await fixture(
          fixtureSource("native.test-17.txt"),
          undefined,
          "",
          undefined,
          {
            allowSignal: true,
            watchdogMs: 2500,
          },
        );
        expect(result.signal).toBe("SIGKILL");
        expect(result.marker).toBe("continued");
        expect(result.forcedCleanup).toBe(true);
      },
    );
  },
);
