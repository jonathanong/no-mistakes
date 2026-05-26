import { coreFn5 } from '@fixture/core/core-5.mts';
import { dataRecord5 } from '@fixture/data/records/data-5.mts';
export function service5(id: string) { return { id, core: coreFn5(), data: dataRecord5 }; }
