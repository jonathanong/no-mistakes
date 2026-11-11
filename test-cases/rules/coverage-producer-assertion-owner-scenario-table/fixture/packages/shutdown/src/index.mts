import * as log from 'fixture-logger';
import { createRegistry } from './registry.mts';
const registry = createRegistry(log);
export const registerA = registry.registerA;
export const registerB = registry.registerB;
export async function gracefulShutdown(deps: any = {}) {
  const errors = await registry.close(deps);
  for (const error of errors) deps.onError?.(error);
}
