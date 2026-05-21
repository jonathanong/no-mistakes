import { coreFn12 } from '@fixture/core/core-12.mts';
import { dataRecord11 } from './data-11.mts';
export interface DataRecord12 { id: string; value: string; }
export const dataRecord12: DataRecord12 = { id: 'data-12', value: coreFn12() };
