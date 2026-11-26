const { spawnSync } = require("node:child_process");
const path = require("node:path");

const args = [
  "nextest",
  "run",
  "--locked",
  "-p",
  "no-mistakes",
  "--lib",
  "--no-default-features",
  "--features",
  "test-instrumentation",
  "--no-fail-fast",
  "--retries",
  "0",
];
const SHARD = "postgres-source";

function run(env = process.env, spawn = spawnSync) {
  const wrapper = path.join(__dirname, "standalone-shard-rustc.sh");
  const childEnv = {
    ...env,
    CARGO_BUILD_JOBS: env.CARGO_BUILD_JOBS || "1",
    CARGO_INCREMENTAL: "0",
    CARGO_PROFILE_DEV_CODEGEN_UNITS: "4096",
    CARGO_PROFILE_TEST_CODEGEN_UNITS: "4096",
    CARGO_PROFILE_DEV_DEBUG: "0",
    CARGO_PROFILE_TEST_DEBUG: "0",
    NM_TEST_SHARD: SHARD,
    NM_INNER_RUSTC_WRAPPER:
      env.RUSTC_WRAPPER && env.RUSTC_WRAPPER !== wrapper ? env.RUSTC_WRAPPER : "",
    RUSTC_WRAPPER: wrapper,
  };
  console.log(`PostgreSQL source identities: compile slice ${SHARD}`);
  const child = spawn(
    "cargo",
    [...args, "--build-jobs", childEnv.CARGO_BUILD_JOBS, "codebase::postgres::source::"],
    { env: childEnv, stdio: "inherit" },
  );
  if (child.error) throw child.error;
  return child.status ?? 1;
}
if (require.main === module) process.exitCode = run();
module.exports = { run };
