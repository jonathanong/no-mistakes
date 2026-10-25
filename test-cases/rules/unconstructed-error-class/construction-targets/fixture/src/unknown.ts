import { WorkerEntrypoint } from 'cloudflare:workers';
export class Worker extends WorkerEntrypoint {}
export function make(Constructor: new () => unknown) {
  return new Constructor();
}
