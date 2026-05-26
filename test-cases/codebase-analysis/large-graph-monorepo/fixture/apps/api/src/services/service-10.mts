import { coreFn10 } from '@fixture/core/core-10.mts';
import { dataRecord10 } from '@fixture/data/records/data-10.mts';
export function service10(id: string) { return { id, core: coreFn10(), data: dataRecord10 }; }
