import { helper } from './helper.mts';
export function service(items: number[]) { return items.map(item => ({ ...helper(item), value: item + 1 })); }
export const callback = (value: number) => service([value]);
