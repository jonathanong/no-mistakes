import { beginTransaction } from '../db/index';

export async function run() {
  const tx = await beginTransaction();
  await tx('DROP TABLE app.factory_proven'); // declared factory DROP
  await tx('SELECT 1'); // declared factory safe
}
