import { openTransaction, other } from "../transaction";
import type { TxExecutor } from "../transaction";

export async function pending(run: TxExecutor) {
  const tx = await openTransaction();
  const plain = openTransaction();
  await tx(`SELECT id FROM relative_pending`);
  await plain(`SELECT id FROM relative_plain`);
  return run(`SELECT id FROM relative_typed`);
}

export async function union(run: TxExecutor | string) {
  return run(`SELECT id FROM relative_union`);
}

export async function inline({ run }: { run: TxExecutor }) {
  return run(`SELECT id FROM relative_inline`);
}

export function qualified(run: Ns.TxExecutor) {
  return run(`SELECT id FROM relative_qualified`);
}

export async function notFactory() {
  const tx = await openTransaction.member();
  return tx(`SELECT id FROM relative_member_factory`);
}
