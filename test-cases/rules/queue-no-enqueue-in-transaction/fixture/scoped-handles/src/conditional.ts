import { beginTransaction } from "@sample/transactions";

export async function maybeCommit(ready: boolean) {
  const tx = await beginTransaction();
  if (ready) {
    await tx.commit();
  }
}
