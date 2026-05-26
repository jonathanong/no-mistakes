import { coreFn9 } from '@fixture/core/core-9.mts';
import { dataRecord8 } from './data-8.mts';
export interface DataRecord9 { id: string; value: string; }
export const dataRecord9: DataRecord9 = { id: 'data-9', value: coreFn9() };
