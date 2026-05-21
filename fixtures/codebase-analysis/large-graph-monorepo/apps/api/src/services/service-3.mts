import { coreFn3 } from '@fixture/core/core-3.mts';
import { dataRecord3 } from '@fixture/data/records/data-3.mts';
export function service3(id: string) { return { id, core: coreFn3(), data: dataRecord3 }; }
