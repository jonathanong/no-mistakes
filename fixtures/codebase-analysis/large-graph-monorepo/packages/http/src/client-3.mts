import { apiPrefix } from '@fixture/config';
import { dataRecord3 } from '@fixture/data/records/data-3.mts';
export async function clientCall3() {
  await fetch(`${apiPrefix}/resource-3/3`);
  return dataRecord3;
}
