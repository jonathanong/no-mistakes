import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";
import { enqueue as previewEnqueue } from "@sample/preview";

function afterCommit(_callback: () => Promise<void>) {}

type SqlTag = (strings: TemplateStringsArray, ...values: unknown[]) => Promise<unknown>;
type TransactionHandle = {
  insertReceipt: () => Promise<string>;
  sql: SqlTag;
};

await transaction("write", async (tx: TransactionHandle) => {
  await tx.insertReceipt();
  await tx.sql`INSERT INTO receipts (id) VALUES (${"member-tagged"})`;
  await previewEnqueue({ receiptId: "preview" });
  afterCommit(async () => {
    await enqueue({ receiptId: "deferred" });
  });
});

await transaction("write", async (tx: SqlTag) => tx`INSERT INTO receipts (id) VALUES (${"bare-tagged"})`);

async function preview() {
  async function enqueue(_job: { receiptId: string }) {}
  await transaction("write", async () => {
    await enqueue({ receiptId: "shadowed" });
  });
}

async function runPreview() {
  async function transaction(_mode: string, callback: () => Promise<void>) {
    await callback();
  }
  await transaction("read", async () => {
    await enqueue({ receiptId: "unrelated-transaction" });
  });
}

const DeferredReceipt = await transaction("write", () => {
  return class ReceiptJob {
    queued = enqueue({ receiptId: "after-construction" });
    tagged = enqueue`after-construction`;
  };
});
new DeferredReceipt();

await transaction("write", () => enqueue.bind(null));

await transaction("write", () => "committed").then(
  () => {},
  () => enqueue({ receiptId: "after-rollback" }),
);

export { preview, runPreview };
