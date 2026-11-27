const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { profileArgs } = require("./native-rust-tests.cjs");
const { cliFixture, napiFixture, platforms } = require("./smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");

function platform(os) {
  const entry = platforms[os];
  if (!entry) throw new Error(`unsupported smoke OS: ${os}`);
  return entry;
}

function escape(name) {
  return name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function cargoCommands(os) {
  const { label, rust } = platform(os);
  return rust.map(({ package: pkg = "no-mistakes", selector, args }) => [
    "test",
    ...profileArgs(label, pkg),
    "--locked",
    "-p",
    pkg,
    ...selector,
    ...args,
  ]);
}

// One --no-run invocation over every listed target, in the same profile and
// package selection as cargoCommands, so the test step compiles nothing.
function buildCommand(os) {
  const { label, rust } = platform(os);
  const packages = new Set(rust.map(({ package: pkg = "no-mistakes" }) => pkg));
  if (packages.size !== 1) throw new Error(`smoke ${os} rust entries must share one package`);
  const [pkg] = packages;
  const selectors = new Map(rust.map(({ selector }) => [selector.join(" "), selector]));
  return [
    "test",
    ...profileArgs(label, pkg),
    "--locked",
    "-p",
    pkg,
    "--no-run",
    ...[...selectors.values()].flat(),
  ];
}

function vitestArgs(os) {
  const { api } = platform(os);
  const names = api.flatMap((entry) => entry.names);
  return [
    "run",
    ...api.map((entry) => entry.file),
    `--testNamePattern=^(?:${names.map(escape).join("|")})$`,
    "--reporter=default",
    "--coverage=false",
  ];
}

// A cargo filter that matches nothing exits 0 with "running 0 tests".
function assertCargoTestsRan(args, output) {
  const ranTests = [...output.matchAll(/^running (\d+) tests?\r?$/gm)].some(
    ([, count]) => Number(count) > 0,
  );
  if (!ranTests) throw new Error(`cargo ${args.join(" ")} ran zero tests; update the smoke list`);
}

function execute(command, args, env = process.env) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} ${args[0]} failed: ${result.status}`);
}

function cargo(args) {
  const result = spawnSync("cargo", args, { cwd: root, encoding: "utf8", maxBuffer: 1 << 28 });
  if (result.error) throw result.error;
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  if (result.status !== 0) throw new Error(`cargo ${args[0]} failed: ${result.status}`);
  assertCargoTestsRan(args, result.stdout);
}

function assertCliResult({ status, stdout, stderr }) {
  if (status !== 1)
    throw new Error(`check exited ${status}, expected a finding. ${stderr}`.trimEnd());
  const report = JSON.parse(stdout);
  const found =
    Array.isArray(report?.rules) &&
    report.rules.some(
      (finding) =>
        finding?.rule === "agents-md-max-size" &&
        finding.file === "AGENTS.md" &&
        finding.line === 1,
    );
  if (!found) throw new Error(`check did not report agents-md-max-size at AGENTS.md:1\n${stdout}`);
}

function cli(binary) {
  execute(binary, ["--version"]);
  const result = spawnSync(
    binary,
    ["check", "--format", "json", "--root", path.join(root, cliFixture)],
    { cwd: root, encoding: "utf8" },
  );
  if (result.error) throw result.error;
  assertCliResult(result);
}

// Order-independent: the dependency API tests in real-napi-api.test.js depend
// on which test first sizes the Rayon pool, so they only run in the full suite.
async function napi(addonPath) {
  process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH = path.resolve(addonPath);
  const api = require(path.join(root, "packages/no-mistakes/index.js"));
  const result = await api.analyzeProject({
    root: path.join(root, napiFixture),
    reports: [{ type: "dependencies", files: ["entry.ts"], relationships: ["import"] }],
  });
  const files = result.reports[0].result.files;
  if (!Array.isArray(files) || files.length === 0)
    throw new Error("analyzeProject returned no dependency files for the saved fixture");
}

function runApi(os, addonPath) {
  const vitest = path.join(path.dirname(require.resolve("vitest/package.json")), "vitest.mjs");
  execute(process.execPath, [vitest, ...vitestArgs(os)], {
    ...process.env,
    NO_MISTAKES_TEST_NAPI_ADDON_PATH: path.resolve(addonPath),
  });
}

function main(os, mode, argument) {
  if (mode === "napi") return napi(argument);
  if (mode === "cli") return cli(argument);
  if (mode === "rust") return cargoCommands(os).forEach(cargo);
  if (mode === "rust-build") return execute("cargo", buildCommand(os));
  if (mode === "api") return runApi(os, argument);
  throw new Error(
    "usage: smoke.cjs <macos|windows> <cli <binary>|rust|rust-build|api <addon>|napi <addon>>",
  );
}

if (require.main === module) {
  Promise.resolve()
    .then(() => main(process.argv[2], process.argv[3], process.argv[4]))
    .catch((error) => {
      console.error(error.message);
      process.exitCode = 1;
    });
}

module.exports = {
  assertCargoTestsRan,
  assertCliResult,
  buildCommand,
  cargoCommands,
  escape,
  vitestArgs,
};
