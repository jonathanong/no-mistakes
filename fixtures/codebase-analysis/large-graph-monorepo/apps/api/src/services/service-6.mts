import { coreFn6 } from '@fixture/core/core-6.mts';
import { dataRecord6 } from '@fixture/data/records/data-6.mts';
export function service6(id: string) { return { id, core: coreFn6(), data: dataRecord6 }; }
