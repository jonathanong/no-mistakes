import { apiPrefix } from '@fixture/config';
import { dataRecord5 } from '@fixture/data/records/data-5.mts';
export async function clientCall5() {
  await fetch(`${apiPrefix}/resource-5/5`);
  return dataRecord5;
}
