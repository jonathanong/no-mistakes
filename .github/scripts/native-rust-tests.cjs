const { spawnSync } = require("node:child_process");

function profileArgs(platform) {
  return platform === "macOS"
    ? [
        "--config",
        "profile.dev.package.no-mistakes.codegen-units=64",
        "--config",
        "profile.test.package.no-mistakes.codegen-units=64",
      ]
    : [];
}

function targets(platform) {
  const shared = [
    {
      selector: ["--test", "suite_cli_postgres"],
      cases: [["cli_postgres_snapshot_"], ["cli_mrf_snapshot_"]],
    },
    {
      selector: ["--test", "suite_cli_package_config"],
      cases: [["cli_pkgcfg_snapshot_"], ["cli_rrb_snapshot_"]],
    },
  ];
  if (platform === "Windows")
    return [
      {
        selector: ["--test", "windows_job_object"],
        cases: [["--", "--exact", "--test-threads=1"]],
      },
      ...shared,
    ];
  if (platform === "macOS")
    return [
      { selector: ["--test", "macos_invocation"], cases: [[]] },
      { selector: ["--bin", "no-mistakes"], cases: [["invocation::tests::blocked_check::"]] },
      ...shared,
    ];
  throw new Error(`unsupported native test platform: ${platform}`);
}

function commands(mode, platform) {
  const selected = targets(platform);
  const base = ["test", ...profileArgs(platform), "--locked", "-p", "no-mistakes"];
  if (mode === "build" || mode === "archive-build")
    return selected.map(({ selector }) => [
      ...base,
      "--no-run",
      "--timings",
      "--verbose",
      ...(mode === "archive-build" ? ["--message-format=json-render-diagnostics"] : []),
      ...selector,
    ]);
  if (mode === "run")
    return selected.flatMap(({ selector, cases }) =>
      cases.map((args) => [...base, ...selector, ...args]),
    );
  throw new Error("usage: native-rust-tests.cjs <build|run>");
}

function buildInvocation(argv = process.argv, env = process.env) {
  const archive = argv[2] === "--archive-tests";
  const testBuild = argv[2] === "--tests" || archive;
  const args = testBuild
    ? commands(archive ? "archive-build" : "build", env.RUNNER_OS)[Number(argv[3])]
    : [
        "build",
        ...profileArgs(env.RUNNER_OS),
        "--locked",
        "-p",
        "no-mistakes",
        "--timings",
        "--verbose",
        "--message-format=json-render-diagnostics",
      ];
  if (!args) throw new Error("expected a native test target index");
  if (testBuild && env.NO_MISTAKES_BUILD_NAPI !== "1")
    throw new Error("native Rust tests require NO_MISTAKES_BUILD_NAPI=1");
  return {
    directory: require("node:path").resolve(
      testBuild
        ? `target/native-test-build-diagnostics/${Number(argv[3])}`
        : "target/native-build-diagnostics",
    ),
    args,
  };
}

function main(mode) {
  if (process.env.NO_MISTAKES_BUILD_NAPI !== "1")
    throw new Error("native Rust tests require NO_MISTAKES_BUILD_NAPI=1");
  for (const [index, args] of commands(mode, process.env.RUNNER_OS).entries()) {
    console.log(`cargo ${args.join(" ")}`);
    const result =
      mode === "build"
        ? spawnSync(
            process.execPath,
            [require("node:path").join(__dirname, "build-native.cjs"), "--tests", String(index)],
            { stdio: "inherit", env: process.env },
          )
        : spawnSync("cargo", args, { stdio: "inherit", env: process.env });
    if (result.error) throw result.error;
    if (result.status !== 0) return result.status ?? 1;
  }
  return 0;
}

if (require.main === module) {
  try {
    if (process.argv.length !== 3) throw new Error("usage: native-rust-tests.cjs <build|run>");
    process.exitCode = main(process.argv[2]);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
module.exports = { targets, commands, buildInvocation };
