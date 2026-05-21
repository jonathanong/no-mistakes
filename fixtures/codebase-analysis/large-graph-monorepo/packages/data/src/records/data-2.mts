import { coreFn2 } from '@fixture/core/core-2.mts';
import { dataRecord1 } from './data-1.mts';
export interface DataRecord2 { id: string; value: string; }
export const dataRecord2: DataRecord2 = { id: 'data-2', value: coreFn2() };
