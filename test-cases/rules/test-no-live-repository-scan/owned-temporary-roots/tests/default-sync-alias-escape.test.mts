import { test } from 'vitest';
import { readdirSync } from 'node:fs';
import fsA from 'node:fs';
import fsB from 'fs';
declare function mutate(value: unknown): void;
test('unproven', () => {
  mutate(fsB);
  const resource = fsA.mkdtempDisposableSync('/tmp/scan-');
  readdirSync(resource.path);
});
