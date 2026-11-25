import {
  analyzeProject,
  collectDiagnostics,
  queueCheck,
  type BatchedQueueCheckOptions,
  type QueueCheckFinding,
  type QueueCheckOptions,
  type QueueCheckReport,
  type QueueRateLimitFindingKind,
} from "no-mistakes";

const shared: QueueRateLimitFindingKind = "shared-queue-rate-limit";
const perProcess: QueueRateLimitFindingKind = "worker-rate-limit-per-process";
// @ts-expect-error Other queue findings are not rate-limit-specific kinds.
const unrelated: QueueRateLimitFindingKind = "unmatched-worker";
const otherKind: QueueCheckFinding["kind"] = "worker-rate-limit-per-process";

const bareOptions = { root: ".", config: "selected.yml" } satisfies QueueCheckOptions;
const batched: BatchedQueueCheckOptions = { includeDiagnostics: true };
async function checkOverloads() {
  const bare: QueueCheckFinding[] = await queueCheck(bareOptions);
  const detailed: QueueCheckReport = await queueCheck({
    root: ".",
    includeDiagnostics: true,
  });
  const collected = await collectDiagnostics("queueCheck", {
    root: ".",
    includeDiagnostics: true,
  });
  const detailedDiagnostics = collected.result.diagnostics;
  const project = await analyzeProject({
    root: ".",
    reports: [{ type: "queueCheck", ...batched }],
  });
  return { bare, detailed, detailedDiagnostics, project, bareOptions };
}
void [shared, perProcess, unrelated, otherKind, checkOverloads];
