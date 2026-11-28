import { beginTransaction } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function missingClose() {
  const tx = await beginTransaction();
  await tx.insertReceipt();
}

export async function destructuredBegin() {
  const { commit } = await beginTransaction();
  await commit();
}

export async function commitInHelper() {
  async function finish() {
    await tx.commit();
  }
  const tx = await beginTransaction();
  await finish();
}

export async function unknownPublisher(publish: () => Promise<void>) {
  await using tx = await beginTransaction();
  await tx.insertReceipt();
  await publish();
}

export async function deferredPublish() {
  await using tx = await beginTransaction();
  await tx.insertReceipt();
  return () => enqueue({ receiptId: "deferred" });
}
