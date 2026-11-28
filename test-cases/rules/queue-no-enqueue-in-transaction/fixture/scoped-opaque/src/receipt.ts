import { beginTransaction } from "@sample/transactions";
import { createPublisher } from "@sample/jobs";

export async function saveReceipt() {
  const tx = await beginTransaction();
  await createPublisher()({ receiptId: "opaque" });
  await tx.commit();
}
