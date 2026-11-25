import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", async (tx) => {
  const receiptId = await tx.insertReceipt();
  await enqueue({ receiptId });
});
