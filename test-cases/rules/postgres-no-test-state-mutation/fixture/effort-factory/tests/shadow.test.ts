import { beginTransaction } from '../db/index';

export async function run(beginTransaction: () => Promise<(sql: string) => unknown>) {
  const tx = await beginTransaction();
  await tx('DROP TABLE app.factory_shadowed'); // shadowed factory name
}
