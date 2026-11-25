import * as database from "@sample/transactions";
import { enqueue, enqueueFallback } from "@sample/jobs";
import { createRecorder } from "@sample/tools";

declare const maybeBegin: typeof database.beginTransaction | undefined;
declare const maybeEnqueue: typeof enqueue | undefined;
const recorder = createRecorder()();

export async function empty() {}

export async function beginShapes() {
  var mutable = await database.beginTransaction();
  await mutable.commit();
  const idle = 0, grouped = await database.beginTransaction();
  await grouped.rollback();
  const optional = await database.beginTransaction?.();
  await optional?.commit();
  const pending = await (maybeBegin ?? database.beginTransaction)();
  await pending.rollback();
  await recorder.commit();
}

export async function publisherShapes() {
  const tx = await database["beginTransaction"]();
  const receiptId = await tx.insertReceipt();
  const label = await receiptId;
  await (maybeEnqueue ?? enqueue)({ receiptId: label });
  await (enqueueFallback ?? enqueue)({ receiptId: label });
  const later = enqueue.bind(undefined, { receiptId: "bound" });
  await tx.commit();
  await recorder.commit();
  await ({ commit: async () => {} }).commit();
  await later();
}

export async function optionalClose() {
  const tx = await database.beginTransaction();
  await tx?.commit();
}

const shared = await database.beginTransaction();
export { shared };
await shared.commit();
