import { apiPrefix } from '@fixture/config';
import { dataRecord11 } from '@fixture/data/records/data-11.mts';
export async function clientCall11() {
  await fetch(`${apiPrefix}/resource-1/11`);
  return dataRecord11;
}
