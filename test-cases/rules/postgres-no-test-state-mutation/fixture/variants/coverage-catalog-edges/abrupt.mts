import { read } from '@example/db';

function pure(value: string) { return value; }
function recursive(value: string): never { return recursive(value); }
function effectful(value: string) { read(value); }

export function pureThrow(value: string) { throw pure(value); }
export function recursiveThrow(value: string) { throw recursive(value); }
export function effectfulThrow(value: string) { throw effectful(value); }
