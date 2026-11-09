import { pageSize } from './constant.mts';
export function pageClause(): string {
  return `LIMIT ${pageSize()} OFFSET 40`;
}
