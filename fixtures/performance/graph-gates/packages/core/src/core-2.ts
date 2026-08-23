import { coreValue3 } from './core-3';
import type { DataRecord2 } from '@fixture/data/records/data-2';
export const coreValue2 = 'core-2' + coreValue3;
export function coreFn2() { return coreValue2; }
export type CoreRecord2 = DataRecord2;
