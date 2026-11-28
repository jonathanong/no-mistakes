import { enqueue } from "@sample/jobs";
import type { Receipt } from "@sample/models";

export async function publishReconciled(receipt: Receipt) {
  await enqueue({ receiptId: receipt.id });
}
