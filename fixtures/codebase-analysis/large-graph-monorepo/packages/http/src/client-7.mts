import { apiPrefix } from '@fixture/config';
import { dataRecord7 } from '@fixture/data/records/data-7.mts';
export async function clientCall7() {
  await fetch(`${apiPrefix}/resource-7/7`);
  return dataRecord7;
}
