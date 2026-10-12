import { read } from '@data-stores/psql';
export function lookup(item: number) { return read(item); }
