import { coreFn3 } from '@fixture/core/core-3';
export interface DataRecord3 { id: string; value: string; }
export const dataRecord3: DataRecord3 = { id: 'data-3', value: coreFn3() };
