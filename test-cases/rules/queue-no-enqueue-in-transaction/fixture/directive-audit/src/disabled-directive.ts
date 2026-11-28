import { enqueue } from "@sample/jobs";

// no-mistakes-disable-next-line queue-no-enqueue-in-transaction -- Pending rollout.
await enqueue({ receiptId: "queued" });
