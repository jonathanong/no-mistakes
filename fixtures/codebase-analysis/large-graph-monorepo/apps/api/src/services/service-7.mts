import { coreFn7 } from '@fixture/core/core-7.mts';
import { dataRecord7 } from '@fixture/data/records/data-7.mts';
export function service7(id: string) { return { id, core: coreFn7(), data: dataRecord7 }; }
