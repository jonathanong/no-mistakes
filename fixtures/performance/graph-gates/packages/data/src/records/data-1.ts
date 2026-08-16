import { coreFn1 } from '@fixture/core/core-1';
export interface DataRecord1 { id: string; value: string; }
export const dataRecord1: DataRecord1 = { id: 'data-1', value: coreFn1() };
