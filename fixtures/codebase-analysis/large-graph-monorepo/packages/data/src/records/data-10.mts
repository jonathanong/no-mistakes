import { coreFn10 } from '@fixture/core/core-10.mts';
import { dataRecord9 } from './data-9.mts';
export interface DataRecord10 { id: string; value: string; }
export const dataRecord10: DataRecord10 = { id: 'data-10', value: coreFn10() };
