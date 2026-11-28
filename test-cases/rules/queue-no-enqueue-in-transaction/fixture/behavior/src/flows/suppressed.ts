import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", async () => {
  // no-mistakes-disable-next-line queue-no-enqueue-in-transaction -- This enqueue is the sole durable handoff.
  await enqueue({ receiptId: "next-line" });
  await enqueue({ receiptId: "line" }); // no-mistakes-disable-line queue-no-enqueue-in-transaction -- Sole durable handoff.
});
