import type { QueueCheckFinding, QueueWaitFindingKind } from "no-mistakes";

const call: QueueWaitFindingKind = "enqueue-and-wait";
const declaration: QueueWaitFindingKind = "enqueue-and-wait-declaration";
// @ts-expect-error Other queue findings are not wait-specific kinds.
const unrelated: QueueWaitFindingKind = "unmatched-worker";
const otherKind: QueueCheckFinding["kind"] = "custom-queue-finding";
void [call, declaration, unrelated, otherKind];
