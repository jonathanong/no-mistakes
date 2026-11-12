import { test } from 'vitest';
import { readdirSync } from 'node:fs';
test('unproven', async () => {
  const resource = { path: process.cwd() };
  readdirSync((await resource).path);
});
