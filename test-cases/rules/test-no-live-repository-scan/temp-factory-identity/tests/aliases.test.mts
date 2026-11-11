import { test } from 'vitest';
import { mkdtempSync as importedFactory, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';

function localAlias() {
  const importedFactory = (..._args: unknown[]) => process.cwd();
  return importedFactory(join(tmpdir(), 'scan-'));
}
let rebound = importedFactory;
rebound = () => process.cwd();

test('a local alias shadow is a live root', () => {
  readdirSync(localAlias());
});
test('a rebound alias is not the Node factory', () => {
  readdirSync(rebound(join(tmpdir(), 'scan-')));
});
test('the original imported factory is owned', () => {
  readdirSync(importedFactory(join(tmpdir(), 'scan-')));
});
