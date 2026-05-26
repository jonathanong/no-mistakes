import { apiPrefix } from '@fixture/config';
import { dataRecord12 } from '@fixture/data/records/data-12.mts';
export async function clientCall12() {
  await fetch(`${apiPrefix}/resource-2/12`);
  return dataRecord12;
}
