import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, rmSync, symlinkSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const directory = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(directory, "..");
const require = createRequire(import.meta.url);
const fixtures = resolve(
  packageRoot,
  "../../test-cases/eslint-plugin/async-call-disposition/fixture",
);
const aggregateLines = [
  11, 12, 13, 14, 15, 16, 17, 23, 24, 25, 26, 27, 28, 29, 35, 36, 37, 38, 39, 40, 41, 47, 48, 49,
  50, 51, 52, 53, 54, 55, 56, 57, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
];
const range = (first, last) =>
  Array.from({ length: last - first + 1 }, (_, index) => first + index);

export function realDrop12(installed, parserInstalled = true) {
  // Copy the plugin outside the workspace so missing peers are physically absent.
  // Only dependency layout is generated; every analyzed source/config is saved.
  const isolated = mkdtempSync(resolve(tmpdir(), "no-mistakes-drop12-"));
  try {
    const plugin = resolve(isolated, "node_modules/eslint-plugin-no-mistakes");
    mkdirSync(plugin, { recursive: true });
    cpSync(resolve(packageRoot, "src"), resolve(plugin, "src"), { recursive: true });
    cpSync(resolve(packageRoot, "package.json"), resolve(plugin, "package.json"));
    const probe = resolve(isolated, "peer-probe.cjs");
    cpSync(resolve(fixtures, "aggregate-spreads/peer-probe.cjs"), probe);
    if (installed) {
      for (const name of parserInstalled
        ? ["typescript", "@typescript-eslint/parser"]
        : ["typescript"]) {
        const target = resolve(isolated, "node_modules", name);
        mkdirSync(dirname(target), { recursive: true });
        symlinkSync(dirname(require.resolve(`${name}/package.json`)), target, "junction");
      }
    }
    const checkerRequire = createRequire(require.resolve("oxlint-tsgolint/package.json"));
    const checker = checkerRequire.resolve(
      `@oxlint-tsgolint/${process.platform}-${process.arch}/tsgolint${process.platform === "win32" ? ".exe" : ""}`,
    );
    const cases = {
      "aggregate-spreads": { "caller.ts": aggregateLines },
      "promise-handoff-alias": {
        "caller.ts": installed ? [11, 15] : [6, 11, 15],
        "nested-caller.ts": installed ? [9, 13] : [4, 9, 13],
        "nested-consumers.ts": [],
        "consumer.ts": installed ? [6] : [],
        "controls.ts": range(7, 11),
        "intersection-caller.ts": range(installed ? 4 : 3, 13),
        "intersection-consumers.ts": installed ? [5] : [],
      },
      "handoff-data": {
        "accepted.ts": installed ? [17] : [5, 10, 12, 14, 17],
        "controls.ts": installed
          ? [7, 9, 13, 17, 19, 22, 23, 24, 26, 30]
          : [7, 9, 13, 17, 19, 22, 24, 26, 30],
        "depth.ts": [37],
        "dynamic-member.ts": [9],
        "date-prototype.ts": installed ? [3, 4] : [3],
        "object-prototype.ts": installed ? [3, 4] : [3],
        "shadowed-date.ts": installed ? [3, 4] : [3],
        "array-prototype.ts": installed ? [3, 7] : [3],
        "pipeline.ts": installed ? [37] : [],
        "snapshot-coercion.ts": [3],
        "snapshot-parameters.ts": installed ? [12, 18, 24, 30] : [5, 12, 18, 24, 30],
      },
    };
    for (const [name, files] of Object.entries(cases)) {
      const root = resolve(isolated, name);
      cpSync(resolve(fixtures, name), root, { recursive: true });
      const result = spawnSync(
        process.execPath,
        [
          resolve(directory, "../../../node_modules/oxlint/bin/oxlint"),
          "--type-aware",
          "--deny-warnings",
          "--format",
          "json",
          "--config",
          resolve(root, ".oxlintrc-isolated.json"),
          ...Object.keys(files).map((file) => resolve(root, file)),
        ],
        {
          cwd: root,
          encoding: "utf8",
          env: {
            ...process.env,
            NODE_PATH: "",
            NODE_OPTIONS: `--require=${JSON.stringify(probe)}`,
            NO_MISTAKES_TEST_PEERS: installed
              ? parserInstalled
                ? "present"
                : "typescript"
              : "absent",
            OXLINT_TSGOLINT_PATH: checker,
          },
        },
      );
      assert.equal(result.status, 1, result.stderr || result.stdout);
      assert.deepEqual(
        JSON.parse(result.stdout)
          .diagnostics.map((item) => [
            item.filename.replaceAll("\\", "/").split("/").at(-1),
            item.labels[0].span.line,
            item.code,
          ])
          .sort(),
        Object.entries(files)
          .flatMap(([file, lines]) =>
            lines.map((line) => [file, line, "no-mistakes(async-call-disposition)"]),
          )
          .sort(),
        `${name}, peers=${installed}`,
      );
    }
  } finally {
    rmSync(isolated, { recursive: true, force: true });
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  realDrop12(false);
  realDrop12(true);
  realDrop12(true, false);
  process.stdout.write(
    "Drop12: real type-aware Oxlint passed with both peers, TypeScript only, and no peers\n",
  );
}
