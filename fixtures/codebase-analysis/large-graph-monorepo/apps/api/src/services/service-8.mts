import { coreFn8 } from '@fixture/core/core-8.mts';
import { dataRecord8 } from '@fixture/data/records/data-8.mts';
export function service8(id: string) { return { id, core: coreFn8(), data: dataRecord8 }; }
