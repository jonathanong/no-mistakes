import { beginTransaction } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function saveBoth() {
  const tx = await beginTransaction();
  {
    const tx = await beginTransaction();
    await tx.commit();
  }
  await enqueue({ receiptId: "outer-transaction-is-open" });
  await tx.commit();
}
