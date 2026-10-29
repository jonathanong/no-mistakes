import { beginTransaction } from '../other/index';

export async function run() {
  const tx = await beginTransaction();
  await tx('DROP TABLE app.factory_unconfigured'); // unconfigured relative factory
}
