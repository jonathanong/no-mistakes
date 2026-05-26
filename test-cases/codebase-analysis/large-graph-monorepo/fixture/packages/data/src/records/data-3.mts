import { coreFn3 } from '@fixture/core/core-3.mts';
import { dataRecord2 } from './data-2.mts';
export interface DataRecord3 { id: string; value: string; }
export const dataRecord3: DataRecord3 = { id: 'data-3', value: coreFn3() };
