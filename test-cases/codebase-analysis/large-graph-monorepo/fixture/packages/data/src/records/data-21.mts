import { coreFn21 } from '@fixture/core/core-21.mts';
import { dataRecord20 } from './data-20.mts';
export interface DataRecord21 { id: string; value: string; }
export const dataRecord21: DataRecord21 = { id: 'data-21', value: coreFn21() };
