import { apiPrefix } from '@fixture/config';
import { dataRecord14 } from '@fixture/data/records/data-14.mts';
export async function clientCall14() {
  await fetch(`${apiPrefix}/resource-4/14`);
  return dataRecord14;
}
