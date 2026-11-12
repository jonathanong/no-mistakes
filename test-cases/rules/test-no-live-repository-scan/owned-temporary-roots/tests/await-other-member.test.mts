import { test } from 'vitest';
import { readdirSync } from 'node:fs';
test('unproven', async () => {
  readdirSync((await Promise.resolve({ other: process.cwd() })).other);
});
