import { enqueue } from "@sample/jobs";
import { begin } from "./barrel";

export async function saveReceipt() {
  const tx = await begin();
  await enqueue({ receiptId: "inside" });
  await tx.commit();
}
