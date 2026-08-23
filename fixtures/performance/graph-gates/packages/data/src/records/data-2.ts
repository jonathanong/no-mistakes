import { coreFn2 } from '@fixture/core/core-2';
export interface DataRecord2 { id: string; value: string; }
export const dataRecord2: DataRecord2 = { id: 'data-2', value: coreFn2() };
