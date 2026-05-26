import { coreFn6 } from '@fixture/core/core-6.mts';
import { dataRecord5 } from './data-5.mts';
export interface DataRecord6 { id: string; value: string; }
export const dataRecord6: DataRecord6 = { id: 'data-6', value: coreFn6() };
