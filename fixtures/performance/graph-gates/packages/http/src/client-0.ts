import { apiPrefix } from '@fixture/config';
import { dataRecord0 } from '@fixture/data/records/data-0';
export async function clientCall0() {
  await fetch(`${apiPrefix}/resource-0/0`);
  return dataRecord0;
}
