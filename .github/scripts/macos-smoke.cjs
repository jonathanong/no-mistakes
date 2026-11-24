const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { profileArgs } = require("./native-rust-tests.cjs");
const { api, cliFixture, napiFixture, rust } = require("./macos-smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");

function escape(name) {
  return name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function cargoCommands() {
  return rust.map(({ selector, args }) => [
    "test",
    ...profileArgs("macOS"),
    "--locked",
    "-p",
    "no-mistakes",
    ...selector,
    ...args,
  ]);
}

function vitestArgs() {
  const names = api.flatMap((entry) => entry.names);
  return [
    "run",
    ...api.map((entry) => entry.file),
    `--testNamePattern=^(?:${names.map(escape).join("|")})$`,
    "--reporter=default",
    "--coverage=false",
  ];
}

function execute(command, args, env = process.env) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} ${args[0]} failed: ${result.status}`);
}

function cli(binary) {
  execute(binary, ["--version"]);
  const result = spawnSync(
    binary,
    ["check", "--format", "json", "--root", path.join(root, cliFixture)],
    { cwd: root, encoding: "utf8" },
  );
  if (result.error) throw result.error;
  if (result.status !== 0 && result.status !== 1)
    throw new Error(`check exited ${result.status}: ${result.stderr}`);
  JSON.parse(result.stdout);
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

function runApi(addonPath) {
  const vitest = path.join(path.dirname(require.resolve("vitest/package.json")), "vitest.mjs");
  execute(process.execPath, [vitest, ...vitestArgs()], {
    ...process.env,
    NO_MISTAKES_TEST_NAPI_ADDON_PATH: path.resolve(addonPath),
  });
}

function main(mode, argument) {
  if (mode === "napi") return napi(argument);
  if (mode === "cli") return cli(argument);
  if (mode === "rust") return cargoCommands().forEach((args) => execute("cargo", args));
  if (mode === "api") return runApi(argument);
  throw new Error("usage: macos-smoke.cjs <cli <binary>|rust|api <addon>|napi <addon>>");
}

if (require.main === module) {
  Promise.resolve()
    .then(() => main(process.argv[2], process.argv[3]))
    .catch((error) => {
      console.error(error.message);
      process.exitCode = 1;
    });
}

module.exports = { cargoCommands, escape, vitestArgs };
