const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { cliFixture, cliCases } = require("./smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");

function cliCargoCommand(os) {
  const cases = cliCases[os];
  if (!Array.isArray(cases)) throw new Error(`unsupported smoke OS: ${os}`);
  return [
    "test",
    "--locked",
    "-p",
    "no-mistakes-invocation",
    "--test",
    "cli_smoke",
    "--",
    "--exact",
    ...cases.map((name) => `cli_invocation::${name}`),
  ];
}

// A cargo filter that matches nothing exits 0 with "running 0 tests".
function assertCargoTestsRan(args, output) {
  const ranTests = [...output.matchAll(/^running (\d+) tests?\r?$/gm)].some(
    ([, count]) => Number(count) > 0,
  );
  if (!ranTests) throw new Error(`cargo ${args.join(" ")} ran zero tests; update the smoke list`);
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
  const result = spawnSync(
    binary,
    ["check", "--format", "json", "--root", path.join(root, cliFixture)],
    { cwd: root, encoding: "utf8" },
  );
  if (result.error) throw result.error;
  assertCliResult(result);
}

function main(os, mode, argument) {
  if (mode === "cli") return cli(argument);
  if (mode === "cli-rust") return cargo(cliCargoCommand(os));
  throw new Error("usage: smoke.cjs <macos|windows> <cli <binary>|cli-rust>");
}

if (require.main === module) {
  try {
    main(process.argv[2], process.argv[3], process.argv[4]);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { assertCargoTestsRan, assertCliResult, cliCargoCommand };
