import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, plugin } from "./helpers.mjs";

const rule = "test-no-wall-time-assertions";

it("snapshots elapsed wall-time assertion diagnostics", async () => {
  const cases = [
    { file: "direct.test.js", count: 2 },
    { file: "captures.spec.ts", count: 2 },
    { file: "imported-clock.test.ts", count: 2 },
    { file: "commonjs-expect.test.cjs", count: 2 },
    { file: "commonjs-timers.test.cjs", count: 0 },
    { file: "commonjs-namespace.test.cjs", count: 0 },
    { file: "commonjs-shadow.test.cjs", count: 1 },
    { file: "scope.test.js", count: 1 },
    { file: "shadowed-expect.test.js", count: 0 },
    { file: "fake-timers.test.ts", count: 0 },
    { file: "namespace-timers.spec.js", count: 0 },
    { file: "global-timers.test.js", count: 0 },
    { file: "unrelated-timers.test.js", count: 1 },
    { file: "suppressions.test.js", count: 1 },
    { file: "suppressed-file.test.js", count: 0 },
    { file: "direct.js", source: "direct.test.js", count: 0 },
    { file: "direct.test.js", options: { allowedPaths: [] }, count: 2 },
    {
      file: "benchmarks/speed.test.js",
      options: { allowedPaths: ["benchmarks/**/*.test.js"] },
      count: 0,
    },
    {
      file: "benchmarks/speed.test.js",
      options: { allowedPaths: ["**/*.bench.js"] },
      count: 1,
    },
  ].map(({ file, source = file, options, count }) => {
    const name = file + (options ? ` (${options.allowedPaths.join(", ")})` : "");
    const sourceType = file.endsWith(".cjs") ? "commonjs" : "module";
    const findings = pluginFindings(
      fixture(`${rule}/${source}`),
      rule,
      options,
      file,
      {},
      sourceType,
    );
    expect(
      findings.map(({ messageId }) => messageId),
      name,
    ).toEqual(Array(count).fill("wallTime"));
    return {
      name,
      file,
      rule,
      options,
      findings,
      ...(sourceType === "commonjs" ? { sourceType } : {}),
    };
  });

  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/test-no-wall-time-assertions.snap",
  );
});

it("exports the rule for opt-in use", () => {
  expect(plugin.rules[rule]).toBeDefined();
  expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
  expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
});
