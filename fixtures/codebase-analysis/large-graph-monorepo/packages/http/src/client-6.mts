import { apiPrefix } from '@fixture/config';
import { dataRecord6 } from '@fixture/data/records/data-6.mts';
export async function clientCall6() {
  await fetch(`${apiPrefix}/resource-6/6`);
  return dataRecord6;
}
