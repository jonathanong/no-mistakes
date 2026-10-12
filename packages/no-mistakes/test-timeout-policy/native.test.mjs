import { fixtureSource } from "../../../fixtures/test-timeout-policy/fixture-source.mjs";
import { describe, expect, it } from "vitest";
import { fixture } from "../../../fixtures/test-timeout-policy/timeout-policy-child.mjs";
describe("native Vitest registration and runtime deadlines", () => {
  it.each([
    `const limit = 30 * 1000 + 1; test('computed', { timeout: limit }, continued)`,
    `const timeout = Number('30001'); describe('inherited', { timeout }, () => test('child', continued))`,
    `const register = test.concurrent; register('alias', continued, 30001)`,
    `test.each([1])('curried %s', { timeout: 30001 }, continued)`,
    `test.runIf(true)('conditional', continued, 30001)`,
    `test.extend('sample', {}, () => 1)('fixture', continued, 30001)`,
    `const child = test.extend({ sample: 1 }); child.beforeAll(() => {}, null); child('child', continued)`,
    `beforeAll(() => {}, '30001'); test('child', continued)`,
    `beforeAll(() => {}, 0); test('child', continued)`,
    `afterAll(() => {}, Infinity); test('child', continued)`,
    `aroundAll(async run => run(), 30001); test('child', continued)`,
    `const around = aroundEach; around(async run => run(), 0); test('child', continued)`,
  ])("rejects during collection before the body: %s", async (source) => {
    const result = await fixture(source);
    expect(result.signal).toBeNull();
    expect(result.code).toBe(1);
    expect(result.output).toContain("must be positive, finite and at most 30000 ms");
    expect(result.marker).toBeUndefined();
  });
  it.each(["30001", "0", "-1", "NaN", "Infinity"])(
    "rejects opaque runtime configuration through a setter alias: %s",
    async (timeout) => {
      const result = await fixture(`test('runtime', () => {
        const { setConfig } = vi
        const options = JSON.parse('{"testTimeout":1}')
        options.testTimeout = ${timeout}
        setConfig(options)
        continued()
      })`);
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.output).toContain("must be positive, finite and at most 30000 ms");
      expect(result.marker).toBeUndefined();
    },
  );
  it.each(["0", "-1", "NaN", "Infinity", "30001"])(
    "rejects a resolved standalone project timeout before collection: %s",
    async (timeout) => {
      const result = await fixture(
        fixtureSource("native.test-6.txt"),
        `{ testTimeout: ${timeout}, hookTimeout: 250 }`,
      );
      expect(result.signal).toBeNull();
      expect(result.code).toBe(1);
      expect(result.marker).toBeUndefined();
    },
  );
  it("guards context completion hooks before their timers are created", async () => {
    const result = await fixture(fixtureSource("native.test-7.txt"));
    expect(result.code).toBe(1);
    expect(result.signal).toBeNull();
    expect(result.output).toContain("must be positive, finite and at most 30000 ms");
    expect(result.marker).toBeUndefined();
  });
  it("rejects raw native config writes through the runner configuration", async () => {
    const result = await fixture(fixtureSource("native.test-8.txt"));
    expect(result.code).toBe(1);
    expect(result.signal).toBeNull();
    expect(result.output).toContain("must be positive, finite and at most 30000 ms");
    expect(result.marker).toBeUndefined();
  });
  it.each([
    `native.test('raw imported test', { timeout: Number('30001') }, continued)`,
    `native.describe('raw inherited suite', { timeout: 30001 }, () => native.test('child', continued))`,
    `const register = native.test.concurrent; register('raw alias', continued, 0)`,
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
  it.each([
    `result.task.timeout = 30001`,
    `const original = result.task; result.task = { ...original, timeout: 250 }; original.timeout = 30001`,
  ])("rejects a delegated extension invalidating the original task: %s", async (mutation) => {
    const result = await fixture(
      fixtureSource("native.test-9.txt"),
      undefined,
      `writeFileSync('extension.marker', 'called'); ${mutation}`,
    );
    expect(result.signal).toBeNull();
    expect(result.code).toBe(1);
    expect(result.output).toContain(
      "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
    );
    expect(result.extensionMarker).toBe("called");
    expect(result.marker).toBeUndefined();
  });
  it("rejects an invalid raw deadline before an extension can mask it", async () => {
    const result = await fixture(
      fixtureSource("native.test-10.txt"),
      undefined,
      `writeFileSync('extension.marker', 'called'); result.task.timeout = 250`,
    );
    expect(result.signal).toBeNull();
    expect(result.code).toBe(1);
    expect(result.output).toContain(
      "Vitest resolved test timeout must be positive, finite and at most 30000 ms",
    );
    expect(result.extensionMarker).toBeUndefined();
    expect(result.marker).toBeUndefined();
  });
  it.each([false, true])(
    "routes package and path aliases with preserveSymlinks=%s",
    async (preserveSymlinks) => {
      for (const specifier of ["deadline-helper", "registration-alias"]) {
        const result = await fixture(
          `import { register } from '${specifier}'; register(continued)`,
          undefined,
          "",
          {
            preserveSymlinks,
            source: `
          import { aroundAll } from 'vitest'
          export function register(body) { aroundAll(async run => run(), Number('30001')) }
        `,
          },
        );
        expect(result.signal).toBeNull();
        expect(result.code).toBe(1);
        expect(result.output).toContain(
          "hook timeout must be positive, finite and at most 30000 ms",
        );
        expect(result.marker).toBeUndefined();
      }
    },
  );
  it("preserves around-hook execution with the composed context guard", async () => {
    const result = await fixture(fixtureSource("native.test-11.txt"));
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it.each([false, true])(
    "preserves legal package helpers with symlinks=%s",
    async (preserveSymlinks) => {
      const result = await fixture(fixtureSource("native.test-12.txt"), undefined, "", {
        preserveSymlinks,
        source: `
        import { test, aroundAll } from 'vitest'
        export function register(body) {
          aroundAll(async run => run(), 30000)
          test('helper body', body, 30000)
        }
      `,
      });
      expect(result.code, result.output).toBe(0);
      expect(result.marker).toBe("continued");
      expect(result.forcedCleanup).toBe(false);
    },
  );
  it.each(["aroundAll", "aroundEach"])("retains native %s short-hang termination", async (hook) => {
    const result = await fixture(`${hook}(async () => { await new Promise(() => {}) }, 25);
      test('unreachable', continued)`);
    expect(result.code).toBe(1);
    expect(result.signal).toBeNull();
    expect(result.output).toMatch(/timed out/i);
    expect(result.marker).toBeUndefined();
    expect(result.forcedCleanup).toBe(false);
  });
  it("retains native short-hang termination", async () => {
    const result = await fixture(fixtureSource("native.test-13.txt"));
    expect(result.signal).toBeNull();
    expect(result.code).toBe(1);
    expect(result.output).toMatch(/timed out/i);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("accepts the inclusive configured ceiling and aliases", async () => {
    const result = await fixture(fixtureSource("native.test-14.txt"));
    expect(result.signal).toBeNull();
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("forwards exactly the validated options snapshot", async () => {
    const result = await fixture(fixtureSource("native.test-15.txt"));
    expect(result.signal).toBeNull();
    expect(result.code).toBe(1);
    expect(result.output).toMatch(/timed out/i);
    expect(result.marker).toBe("continued");
  });
  it("preserves an undefined optional deadline and typed fixture data", async () => {
    const result = await fixture(fixtureSource("native.test-16.txt"));
    expect(result.signal).toBeNull();
    expect(result.code, result.output).toBe(0);
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(false);
  });
  it("uses separate outer containment for a blocked event loop", { timeout: 10000 }, async () => {
    const result = await fixture(fixtureSource("native.test-17.txt"));
    expect(result.signal).toBe("SIGKILL");
    expect(result.marker).toBe("continued");
    expect(result.forcedCleanup).toBe(true);
  });
});
