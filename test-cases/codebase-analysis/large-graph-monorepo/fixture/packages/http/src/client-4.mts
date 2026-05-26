import { apiPrefix } from '@fixture/config';
import { dataRecord4 } from '@fixture/data/records/data-4.mts';
export async function clientCall4() {
  await fetch(`${apiPrefix}/resource-4/4`);
  return dataRecord4;
}
