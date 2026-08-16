import { coreFn1 } from '@fixture/core/core-1';
import { dataRecord1 } from '@fixture/data/records/data-1';
export function service1(id: string) { return { id, core: coreFn1(), data: dataRecord1 }; }
