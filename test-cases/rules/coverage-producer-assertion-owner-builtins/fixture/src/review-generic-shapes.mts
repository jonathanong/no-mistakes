import { AsyncLocalStorage } from 'node:async_hooks';
import { owner } from './owner.mjs';

type Result = { value: string };
const context = new AsyncLocalStorage<Result>();
const PARTITIONS = ['ALPHA', 'BETA'] as const;

function invoke(handler: () => Result): Result {
  return context.run({ value: '' }, handler)!;
}

export function scriptRun() {
  return invoke(() => {
    let input;
    input.toFixed(1);
    PARTITIONS.find((label) => label.toLowerCase() === 'alpha');
    const joined = PARTITIONS.join(',');
    const values = new Set(['ALPHA']);
    values.has('ALPHA');
    return { value: joined };
  });
}

export function explicitUnknown() {
  const input: unknown = 'x';
  input.toFixed(1);
}

const custom = { join() { owner(); } };

export function customMethod() {
  custom.join();
}

export function wrongKnownReceiver() {
  const values: Set<string> = new Set(['ALPHA']);
  values.map(owner);
}

export function unresolvedWithoutCaller(callback: (value: number) => void) {
  callback(1);
}
