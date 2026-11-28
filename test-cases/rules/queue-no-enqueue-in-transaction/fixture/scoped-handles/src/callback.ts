import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", async (tx) => {
  const receiptId = await tx.insertReceipt();
  await enqueue({ receiptId });
});

await transaction("write", async (tx) => {
  [0].forEach(async () => {
    await tx.insertReceipt();
    await enqueue({ receiptId: "async-callback" });
  });
});
