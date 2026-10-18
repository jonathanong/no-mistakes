import { openTransaction, other } from "../transaction";
import type { TxExecutor } from "../transaction";

export async function pending(run: TxExecutor) {
  const tx = await openTransaction();
  const plain = openTransaction();
  const alias = tx;
  await tx(`SELECT id FROM relative_pending`);
  await tx.query(`SELECT id FROM relative_member_factory`);
  await alias.query(`SELECT id FROM relative_member_alias`);
  await plain(`SELECT id FROM relative_plain`);
  return run(`SELECT id FROM relative_typed`);
}

export function member(run: TxExecutor) {
  const alias = run;
  run.query(`SELECT id FROM relative_member_type`);
  {
    // A nested HTTP-style `run` must not inherit the typed executor's owner.
    const run = { query: (_params: object) => undefined };
    run.query({ resource: 'not_sql' });
  }
  return alias.query(`SELECT id FROM relative_member_type_alias`);
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
