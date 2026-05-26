import { apiPrefix } from '@fixture/config';
import { dataRecord15 } from '@fixture/data/records/data-15.mts';
export async function clientCall15() {
  await fetch(`${apiPrefix}/resource-5/15`);
  return dataRecord15;
}
