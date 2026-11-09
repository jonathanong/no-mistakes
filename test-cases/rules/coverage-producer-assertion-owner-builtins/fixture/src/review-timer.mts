import { strictEqual } from 'node:assert/strict';

if (import.meta.main) {
  setTimeout(() => strictEqual(1, 1), 0);
}

export function shadowedTimer(setTimeout: (callback: () => void) => void) {
  setTimeout(() => strictEqual(1, 1), 0);
}
