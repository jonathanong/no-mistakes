import { apiPrefix } from '@fixture/config';
import { dataRecord0 } from '@fixture/data/records/data-0';
export async function clientCall0() {
  await fetch("/api/v1/resource-0/0");
  return { apiPrefix, dataRecord0 };
}
