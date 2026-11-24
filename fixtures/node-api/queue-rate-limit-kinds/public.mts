import type { QueueCheckFinding, QueueRateLimitFindingKind } from "no-mistakes";

const shared: QueueRateLimitFindingKind = "shared-queue-rate-limit";
const perProcess: QueueRateLimitFindingKind = "worker-rate-limit-per-process";
// @ts-expect-error Other queue findings are not rate-limit-specific kinds.
const unrelated: QueueRateLimitFindingKind = "unmatched-worker";
const otherKind: QueueCheckFinding["kind"] = "worker-rate-limit-per-process";
void [shared, perProcess, unrelated, otherKind];
