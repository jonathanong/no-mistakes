import { coreFn2 } from '@fixture/core/core-2';
import { dataRecord2 } from '@fixture/data/records/data-2';
export function service2(id: string) { return { id, core: coreFn2(), data: dataRecord2 }; }
