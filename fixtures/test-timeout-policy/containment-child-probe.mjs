import { EventEmitter } from 'node:events';

/** Portable observation probe: no spawned process or platform process group. */
export function containmentChildProbe() {
  const child = new EventEmitter();
  child.stdout = new EventEmitter();
  child.stderr = new EventEmitter();
  return child;
}
