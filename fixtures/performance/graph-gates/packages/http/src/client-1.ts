import { apiPrefix } from '@fixture/config';
import { dataRecord1 } from '@fixture/data/records/data-1';
export async function clientCall1() {
  await fetch(`${apiPrefix}/resource-1/1`);
  return dataRecord1;
}
