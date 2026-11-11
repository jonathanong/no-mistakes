import { test } from 'vitest';
import { readdirSync } from 'node:fs';

test('a computed direct factory selector is unresolved', () => {
  readdirSync(require('fs')['mkdtempSync']('scan-'));
});
