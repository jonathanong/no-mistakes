import { begin as start, send as publish } from "./handle-barrel";

export async function saveReceipt() {
  const tx = await start();
  await publish({ receiptId: "inside" });
  await tx.commit();
  await publish({ receiptId: "after-commit" });
}
