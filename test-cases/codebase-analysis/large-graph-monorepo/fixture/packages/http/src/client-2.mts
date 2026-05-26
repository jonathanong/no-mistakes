import { apiPrefix } from '@fixture/config';
import { dataRecord2 } from '@fixture/data/records/data-2.mts';
export async function clientCall2() {
  await fetch(`${apiPrefix}/resource-2/2`);
  return dataRecord2;
}
