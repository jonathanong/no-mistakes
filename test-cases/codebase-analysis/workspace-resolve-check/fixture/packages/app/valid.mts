import '@fx/lib/target';
import type { Value } from '@fx/lib/types.d';
await import('@fx/main');
export async function load() { return import('@fx/lib/target'); }
export type { Value };
