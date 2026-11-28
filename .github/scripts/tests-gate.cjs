const rustJobs = new Set([
  "docs-tests",
  "tests",
  "standalone-build",
  "standalone-shards",
  "standalone-pr",
]);
const macosJobs = new Set(["native-rust-tests-macos", "native-api-tests-macos"]);
const windowsJobs = new Set([
  "native-test-build",
  "native-rust-tests",
  "native-api-tests",
]);
const jobs = JSON.parse(process.env.RESULTS);
const mainPush = process.env.MACOS_EVENT === "push" && process.env.MACOS_REF === "refs/heads/main";

for (const [name, { result }] of Object.entries(jobs)) {
  if (result === "success") continue;
  const macos = macosJobs.has(name);
  const windows = windowsJobs.has(name);
  if (result === "skipped") {
    if (rustJobs.has(name) && process.env.RUST_REQUIRED === "false") continue;
    if (name === "standalone-pr" && process.env.MACOS_EVENT !== "pull_request") continue;
    if (
      (name === "standalone-build" || name === "standalone-shards") &&
      process.env.MACOS_EVENT === "pull_request"
    )
      continue;
    if (macos && process.env.MACOS_REQUIRED === "false") continue;
    if (windows && process.env.WINDOWS_REQUIRED === "false") continue;
  }
  if (mainPush && macos && result === "cancelled") continue;
  console.error(`${name}: ${result}`);
  process.exitCode = 1;
}
