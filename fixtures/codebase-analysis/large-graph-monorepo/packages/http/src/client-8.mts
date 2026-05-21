import { apiPrefix } from '@fixture/config';
import { dataRecord8 } from '@fixture/data/records/data-8.mts';
export async function clientCall8() {
  await fetch(`${apiPrefix}/resource-8/8`);
  return dataRecord8;
}
