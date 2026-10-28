import { beginTransaction } from '../db/index';
declare const opaque: (sql: string) => unknown;

export async function run() {
  let tx = await beginTransaction();
  tx = opaque;
  await tx('DROP TABLE app.factory_mutated'); // mutable factory receiver
}
