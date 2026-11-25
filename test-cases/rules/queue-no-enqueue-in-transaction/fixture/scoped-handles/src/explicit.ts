import { beginTransaction as begin } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function commitReceipt() {
  const tx = await begin();
  const receiptId = await tx.insertReceipt();
  await enqueue({ receiptId });
  await tx.commit();
  await enqueue({ receiptId: "after-commit" });
  return tx;
}

export async function cancelReceipt() {
  let tx = begin();
  await tx.rollback();
  await enqueue({ receiptId: "after-rollback" });
}
