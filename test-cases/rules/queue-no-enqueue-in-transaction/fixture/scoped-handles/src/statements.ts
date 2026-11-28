import { beginTransaction } from "@sample/transactions";
import { enqueue } from "@sample/jobs";

export async function unawaitedClose() {
  const tx = beginTransaction();
  tx.commit();
  await enqueue({ receiptId: "unawaited" });
}

export async function doubleClose() {
  const tx = await beginTransaction();
  await tx.commit();
  await tx.rollback();
  await enqueue({ receiptId: "double" });
}

export async function plainStatements() {
  const tx = await beginTransaction();
  tx;
  await tx.insertReceipt();
  await enqueue({ receiptId: "inside" });
  await tx.commit();
}

export async function nestedUse() {
  const tx = await beginTransaction();
  const run = async () => {
    await tx.insertReceipt();
  };
  await run();
  await tx.commit();
}

export async function awaitedValue(value: Promise<number>) {
  const tx = await beginTransaction();
  await value;
  await tx.commit();
}

export async function builtinShapes(value: Promise<{ id: string }>, items: number[]) {
  const tx = await beginTransaction();
  Math.max(1, 2);
  fetch("https://example.test");
  Array.from(items, () => enqueue({ receiptId: "from" }));
  items.map(() => enqueue({ receiptId: "map" }));
  (await tx.insertReceipt()).toString();
  (await value).id.trim();
  tx.insertReceipt().then(enqueue);
  await tx.commit();
}
