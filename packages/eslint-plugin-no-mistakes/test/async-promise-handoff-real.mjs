import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const testDirectory = dirname(fileURLToPath(import.meta.url));
export const fixture = resolve(
  testDirectory,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture",
);
export const handoff = resolve(fixture, "promise-handoff");
export const callerLines = [18, 23, 28, 32, 36, 49, 51, 59, 61, 63];
export const recallLines = [5, 9, 12, 16, 19, 21, 24, 29, 32, 34, 38];
export const slotLines = [3, 5, 7, 10, 12, 14, 16, 18, 20, 22, 24];
export const guardLines = [
  12, 16, 18, 20, 22, 24, 26, 32, 33, 35, 37, 39, 43, 47, 51, 52, 54, 56, 58, 60, 62, 64, 66, 68,
  70, 72, 74, 76, 78, 80, 82, 84, 86, 88, 96, 98, 101, 107, 109, 112, 127, 132, 140, 142, 144, 146,
  148, 150,
];
const ruleCode = "no-mistakes(async-call-disposition)";
const proven = "proven-consumers/";
const cases = [
  {
    config: "promise-handoff-alias/.oxlintrc.json",
    lines: {
      "promise-handoff-alias/caller.ts": [15],
      "promise-handoff-alias/consumer.ts": [6],
      "promise-handoff-alias/controls.ts": [7, 8, 9, 10, 11],
    },
  },
  {
    config: "promise-handoff/.oxlintrc.json",
    files: readdirSync(handoff)
      .filter((name) => name.endsWith(".ts"))
      .map((name) => `promise-handoff/${name}`),
    lines: {
      "promise-handoff/caller.ts": callerLines,
      "promise-handoff/dropped.ts": [4],
      "promise-handoff/typed-recall.ts": recallLines,
      "promise-handoff/guard-caller.ts": guardLines,
      "promise-handoff/slot-guards.ts": slotLines,
      "promise-handoff/parameter-guards.ts": [5],
      "promise-handoff/static-final.ts": [3],
      "promise-handoff/extra-caller.ts": [7, 9, 11, 15, 17],
    },
  },
  {
    config: `${proven}.oxlintrc-proven.json`,
    lines: {
      [`${proven}cross-file.ts`]: [7],
      [`${proven}cross-file-chain.ts`]: [5, 7, 9, 11, 13, 15, 17],
      [`${proven}cross-file-hostile.ts`]: [
        5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35, 37, 39, 41, 44, 47, 50, 53, 56,
      ],
      [`${proven}cross-file-forwarding.ts`]: [6, 7, 8, 9, 10, 11, 12, 13, 16, 18, 20, 21, 22, 24],
      [`${proven}cross-file-receivers.ts`]: [5, 7, 11, 13, 17, 22, 24, 30, 33, 39],
      [`${proven}cross-file-receiver-helper-chain.ts`]: [4, 19, 22, 26, 30, 34, 38, 42, 81, 84, 90],
      [`${proven}sibling-mutations.ts`]: [4, 6, 8, 10, 12, 14, 16, 18, 20, 24, 26, 28],
      [`${proven}cross-file-prototypes.ts`]: [3, 5, 7],
      [`${proven}alias/caller.ts`]: [],
      [`${proven}dependency-copy/caller.ts`]: [
        72, 78, 84, 90, 96, 102, 108, 144, 150, 156, 162, 168, 174, 180, 186, 192, 198, 204, 210,
        216, 222, 228, 234, 240, 252, 264, 270, 276, 282, 288,
      ],
      [`${proven}dependency-copy-spellings/caller.ts`]: [
        38, 50, 56, 68, 80, 86, 98, 104, 116, 122, 134, 140, 146, 152,
      ],
    },
  },
  {
    config: `${proven}.oxlintrc.json`,
    lines: {
      [`${proven}local.ts`]: [16, 19, 25, 27, 31, 38, 43, 50, 56, 64],
      [`${proven}imported.ts`]: [7, 9, 11, 13, 17, 19, 21, 23, 25, 28, 30, 32],
      [`${proven}properties.ts`]: [7, 9, 11, 13, 15, 18, 20, 24, 28],
    },
  },
];

export function runRealGate(typeAware = true) {
  for (const scenario of cases) {
    const paths = scenario.files || Object.keys(scenario.lines);
    for (const name of paths) assert.ok(existsSync(resolve(fixture, name)), name);
    const result = spawnSync(
      process.execPath,
      [
        resolve(testDirectory, "../../../node_modules/oxlint/bin/oxlint"),
        ...(typeAware ? ["--type-aware"] : []),
        "--deny-warnings",
        "--config",
        resolve(fixture, scenario.config),
        "--format",
        "json",
        ...paths.map((name) => resolve(fixture, name)),
      ],
      {
        encoding: "utf8",
        maxBuffer: 8 * 1024 * 1024,
        env: {
          ...process.env,
          NODE_OPTIONS: `${process.env.NODE_OPTIONS || ""} --require=${JSON.stringify(resolve(handoff, "forbid-program.cjs"))}`,
        },
      },
    );
    assert.equal(result.status, 1, result.stderr || result.stdout);
    const diagnostics = JSON.parse(result.stdout).diagnostics;
    const actual = diagnostics
      .map((item) => [
        relative(fixture, resolve(item.filename || ".")).replaceAll("\\", "/"),
        item.labels?.[0]?.span?.line,
        item.code,
      ])
      .sort();
    const expected = Object.entries(scenario.lines)
      .flatMap(([name, lines]) => lines.map((line) => [name, line, ruleCode]))
      .sort();
    assert.deepEqual(actual, expected, scenario.config);
  }
}

// The standalone command is deliberately run before any typed/untyped ESLint test.
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  runRealGate();
  process.stdout.write("Real type-aware Oxlint: handoff and #2740/#2766 recall gates passed\n");
}
