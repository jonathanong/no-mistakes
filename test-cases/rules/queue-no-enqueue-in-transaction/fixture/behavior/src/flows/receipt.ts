import { transaction as runTransaction } from "@sample/database";
import { enqueue as publishReceipt } from "@sample/jobs";
import * as jobs from "@sample/jobs";

const receiptId = await runTransaction("write", async () => {
  await publishReceipt({ receiptId: "draft" });
  return "committed";
});
await runTransaction("write", () => jobs.enqueue({ receiptId }));
await publishReceipt({ receiptId });
