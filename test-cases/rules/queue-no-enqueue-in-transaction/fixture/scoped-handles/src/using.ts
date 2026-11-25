import { beginTransaction } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function saveDraft() {
  {
    await using tx = await beginTransaction();
    const receiptId = await tx.insertReceipt();
    await enqueue({ receiptId });
  }
  await enqueue({ receiptId: "after-disposal" });
}
