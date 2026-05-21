import { apiPrefix } from '@fixture/config';
import { dataRecord10 } from '@fixture/data/records/data-10.mts';
export async function clientCall10() {
  await fetch(`${apiPrefix}/resource-0/10`);
  return dataRecord10;
}
