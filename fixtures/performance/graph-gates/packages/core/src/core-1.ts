import { coreValue2 } from './core-2';
import type { DataRecord1 } from '@fixture/data/records/data-1';
export const coreValue1 = 'core-1' + coreValue2;
export function coreFn1() { return coreValue1; }
export type CoreRecord1 = DataRecord1;
