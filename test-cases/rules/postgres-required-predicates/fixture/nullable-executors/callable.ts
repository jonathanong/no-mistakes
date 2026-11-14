import type { Execute } from 'db-lib';
declare const id: number;
export async function functionAssignment() {
  let execute: Execute | undefined;
  execute = async function (_sql, _binds) { return undefined; };
  await execute('UPDATE items SET a=1 WHERE id=$1', [id]);
  await execute('UPDATE items SET a=2 WHERE id=$1 AND account_id=$2', [id, id]);
}
export async function arrowAssignment() {
  let execute: Execute | undefined;
  execute = async (_sql, _binds) => undefined;
  await execute('UPDATE items SET a=1 WHERE id=$1', [id]);
  await execute('UPDATE items SET a=2 WHERE id=$1 AND account_id=$2', [id, id]);
}
