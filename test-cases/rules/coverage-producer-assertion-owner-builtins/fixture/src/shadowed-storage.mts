import { AsyncLocalStorage as Storage } from 'node:async_hooks';
import { owner } from './owner.mjs';
export function shadowed(Storage: string) { new Storage().run('id', owner); }
