import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prepare, oxlint, hashSources, archiveSources } from "./source-proof-prepare.mjs";
const here = dirname(fileURLToPath(import.meta.url));
const [
  action,
  directory = "/tmp/no-mistakes-2781-benchmark",
  label = "before",
  count = "4096",
  variant = "dense",
  baseline = "HEAD",
] = process.argv.slice(2);
const root = resolve(directory);
if (action === "prepare") {
  console.log(JSON.stringify(prepare(root, Number(count), variant, baseline), null, 2));
} else if (action === "candidate") {
  archiveSources(label, resolve(root, "peers/after/src"));
  console.log(hashSources(resolve(root, "peers/after/src")));
} else {
  const metadata = JSON.parse(readFileSync(resolve(root, "metadata.json"), "utf8"));
  const workload = resolve(root, "workload");
  const mode = label.split("-")[0];
  assert.ok(["before", "after", "absent"].includes(mode));
  const output = resolve(root, "results");
  mkdirSync(output, { recursive: true });
  const env = {
    ...process.env,
    NODE_PATH: "",
    NODE_OPTIONS:
      action === "profile" ? `--require=${JSON.stringify(resolve(workload, "profile.cjs"))}` : "",
    OXLINT_TSGOLINT_PATH: metadata.checker,
  };
  const result = spawnSync(
    "python3",
    [
      resolve(here, "source-proof-measure.py"),
      output,
      label,
      process.execPath,
      oxlint,
      "--type-aware",
      "--deny-warnings",
      "--threads=4",
      "--format=json",
      "--config",
      resolve(workload, `.oxlintrc-${mode}.json`),
      "callers",
      "shared",
    ],
    { cwd: workload, env, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(readFileSync(resolve(output, `${label}.stdout`), "utf8"));
  const diagnostics = report.diagnostics.sort((a, b) =>
    JSON.stringify(a).localeCompare(JSON.stringify(b)),
  );
  const plugin = diagnostics.filter((item) => item.code === "no-mistakes(async-call-disposition)");
  assert.equal(plugin.length, metadata.callers);
  const metrics = JSON.parse(readFileSync(resolve(output, `${label}.metrics.json`), "utf8"));
  assert.equal(metrics.exitCode, 1);
  metrics.diagnosticCount = diagnostics.length;
  const signature = JSON.stringify(diagnostics);
  const expected = resolve(output, "diagnostics.json");
  if (existsSync(expected)) assert.equal(signature, readFileSync(expected, "utf8"));
  else writeFileSync(expected, signature);
  metrics.diagnosticHash = createHash("sha256").update(signature).digest("hex");
  metrics.fixtureHash = hashSources(workload);
  metrics.sourceHash = hashSources(
    resolve(root, mode === "absent" ? "absent/plugin/src" : `peers/${mode}/src`),
  );
  if (action === "profile") {
    metrics.profile = readFileSync(resolve(output, `${label}.stderr`), "utf8")
      .split("\n")
      .filter((line) => line.startsWith("NO_MISTAKES_SOURCE_PROFILE "))
      .map((line) => JSON.parse(line.slice("NO_MISTAKES_SOURCE_PROFILE ".length)));
  }
  writeFileSync(resolve(output, `${label}.metrics.json`), JSON.stringify(metrics, null, 2));
  console.log(JSON.stringify(metrics, null, 2));
}
