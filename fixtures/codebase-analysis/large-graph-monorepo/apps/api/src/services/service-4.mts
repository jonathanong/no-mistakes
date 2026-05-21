import { coreFn4 } from '@fixture/core/core-4.mts';
import { dataRecord4 } from '@fixture/data/records/data-4.mts';
export function service4(id: string) { return { id, core: coreFn4(), data: dataRecord4 }; }
