import { beginTransaction } from "@sample/transactions";

export async function handOffTransaction() {
  const tx = await beginTransaction();
  return tx;
}
