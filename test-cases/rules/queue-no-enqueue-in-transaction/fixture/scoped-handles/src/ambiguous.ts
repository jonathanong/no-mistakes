import { beginTransaction } from "@sample/transactions";
import { enqueue, enqueueFallback } from "@sample/jobs";
import { queueClient, safeClient } from "@sample/tools";

export async function ternaryCallee(flag: boolean) {
  const tx = await beginTransaction();
  await (flag ? enqueue : enqueueFallback)({ receiptId: "ternary" });
  await tx.commit();
}

export async function nullishKnownCallee() {
  const tx = await beginTransaction();
  await (enqueueFallback ?? enqueue)({ receiptId: "nullish-known" });
  await tx.commit();
}

export async function lookupCallees(index: number, key: "a" | "b") {
  const list = [enqueue, enqueueFallback];
  const map = { a: enqueue, b: enqueueFallback };
  const tx = await beginTransaction();
  await list[index]({ receiptId: "array-dynamic" });
  await map[key]({ receiptId: "object-dynamic" });
  await tx.commit();
}

export async function repeatedReceiverPublisher() {
  const tx = await beginTransaction();
  for (const item of ["a", "b"]) {
    queueClient.send({ receiptId: item });
    queueClient.send = enqueue;
  }
  await tx.commit();
}

export async function repeatedReceiverNonPublisher() {
  const tx = await beginTransaction();
  for (const item of ["a", "b"]) {
    safeClient.send({ receiptId: item });
    safeClient.send = enqueueFallback;
  }
  await tx.commit();
}
