import { AsyncLocalStorage as Storage } from 'node:async_hooks';
import { owner } from './owner.mjs';

const instance = new Storage<string>();
export function staticRun() { Storage.run('id', owner); }
export function staticExit() { Storage.exit(owner); }
export function instanceRun() { instance.run('id', owner); }
export function instanceExit() { instance.exit(owner); }
export function newInstanceRun() { new Storage<string>().run('id', owner); }
export function typedInstanceRun(typed: Storage<string>) { typed.run('id', owner); }
export function unknownRun(unknown: { run: (id: string, callback: () => void) => void }) {
  unknown.run('id', owner);
}
