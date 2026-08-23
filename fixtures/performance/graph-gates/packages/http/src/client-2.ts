import { apiPrefix } from '@fixture/config';
import { dataRecord2 } from '@fixture/data/records/data-2';
export async function clientCall2() {
  await fetch("/api/v1/resource-2/2");
  return { apiPrefix, dataRecord2 };
}
