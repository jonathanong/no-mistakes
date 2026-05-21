import { coreFn4 } from '@fixture/core/core-4.mts';
import { dataRecord3 } from './data-3.mts';
export interface DataRecord4 { id: string; value: string; }
export const dataRecord4: DataRecord4 = { id: 'data-4', value: coreFn4() };
