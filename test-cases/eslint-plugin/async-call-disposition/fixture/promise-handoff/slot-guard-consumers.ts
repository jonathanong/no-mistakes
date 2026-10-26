import { enqueueEmail } from '@app/jobs';
import type { Dependencies } from './types';
export function slot(dependencies: Dependencies, _extra?: unknown): Dependencies {
  return dependencies;
}
export function restSlot(dependencies: Dependencies, ..._rest: unknown[]): Dependencies {
  return dependencies;
}
export function reassignedSlot(dependencies: Dependencies): Dependencies {
  dependencies = { x: enqueueEmail };
  return dependencies;
}
export function writtenSlot(dependencies: Dependencies): Dependencies {
  dependencies.x = enqueueEmail;
  return dependencies;
}
export function destructuredSlot({ x }: Dependencies) {
  x([], undefined);
}
