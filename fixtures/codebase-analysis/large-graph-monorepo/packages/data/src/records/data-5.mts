import { coreFn5 } from '@fixture/core/core-5.mts';
import { dataRecord4 } from './data-4.mts';
export interface DataRecord5 { id: string; value: string; }
export const dataRecord5: DataRecord5 = { id: 'data-5', value: coreFn5() };
