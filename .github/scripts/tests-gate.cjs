const rustJobs = new Set(["docs-tests", "tests", "standalone-build", "standalone-shards"]);
const macosJobs = new Set(["native-rust-tests-macos", "native-api-tests-macos"]);
const windowsJobs = new Set([
  "native-build",
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
    if (
      process.env.RUST_REQUIRED === "false" &&
      (rustJobs.has(name) || macos || windows || name === "windows-smoke")
    )
      continue;
    if (macos && process.env.MACOS_REQUIRED === "false") continue;
    if (name === "macos-smoke" && process.env.MACOS_SMOKE_REQUIRED === "false") continue;
    if (windows && process.env.WINDOWS_REQUIRED === "false") continue;
    if (name === "windows-smoke" && process.env.WINDOWS_REQUIRED === "true") continue;
  }
  if (mainPush && macos && result === "cancelled") continue;
  console.error(`${name}: ${result}`);
  process.exitCode = 1;
}
