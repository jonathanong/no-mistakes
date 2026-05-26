import { apiPrefix } from '@fixture/config';
import { dataRecord13 } from '@fixture/data/records/data-13.mts';
export async function clientCall13() {
  await fetch(`${apiPrefix}/resource-3/13`);
  return dataRecord13;
}
