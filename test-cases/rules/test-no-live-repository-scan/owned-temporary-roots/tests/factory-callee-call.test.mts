import { test } from 'vitest';
import { readdirSync } from 'node:fs';
function getFactory() {
  return () => ({ path: process.cwd() });
}
test('unproven', () => {
  readdirSync(getFactory()('/tmp/scan-').path);
});
