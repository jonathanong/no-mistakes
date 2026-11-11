import {
  installBrowserLifecycleReceipts,
  recordBrowserLifecycle,
} from "../browser-lifecycle-probe.mjs";

const mode = process.argv[2];
if (mode.endsWith("-stall")) setTimeout(() => process.exit(99), 8000);
recordBrowserLifecycle("fixture-start", { mode });
process.stderr.write(`fixture-mode:${mode}\n`);
if (mode === "startup-stall") {
  setInterval(() => {}, 1000);
} else {
  const reporter = installBrowserLifecycleReceipts({});
  reporter.onInit({ onClose() {} });
  reporter.onTestRunStart([]);
  reporter.onTestCaseResult({
    name: "saved child",
    result: () => ({ state: mode === "reject" ? "failed" : "passed" }),
  });
  reporter.onTestRunEnd([], [], mode === "reject" ? "failed" : "passed");
  process.stdout.write("FIXTURE_BODY_COMPLETED\n");
  if (mode === "shutdown-stall") setInterval(() => {}, 1000);
  else process.exitCode = mode === "reject" ? 1 : 0;
}
