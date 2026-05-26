import { apiPrefix } from '@fixture/config';
import { dataRecord9 } from '@fixture/data/records/data-9.mts';
export async function clientCall9() {
  await fetch(`${apiPrefix}/resource-9/9`);
  return dataRecord9;
}
