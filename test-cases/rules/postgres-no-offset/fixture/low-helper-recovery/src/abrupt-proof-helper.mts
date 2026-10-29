import { read } from '@example/db';

export function assertNever(value: string): never {
  throw new Error(String(value));
}
export function effectful(value: string): never {
  throw new Error(read(value));
}
export async function asyncNever(value: string): Promise<never> {
  throw new Error(value);
}
