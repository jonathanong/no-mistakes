import { beginTransaction as begin } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function commitReceipt() {
  const tx = await begin();
  const receiptId = await tx.insertReceipt();
  await enqueue({ receiptId });
  type Handle = typeof tx;
  const rows = await tx.selectRows();
  const ids = rows.map((row: { id: string }) => row.id.trim()).join(",");
  await tx.insertInto("receipts").values({ ids }).execute();
  const _typed: Handle | undefined = undefined;
  await tx.commit();
  await enqueue({ receiptId: "after-commit" });
  return { tx, _typed };
}

export async function cancelReceipt() {
  let tx = begin();
  await tx.rollback();
  await enqueue({ receiptId: "after-rollback" });
}
