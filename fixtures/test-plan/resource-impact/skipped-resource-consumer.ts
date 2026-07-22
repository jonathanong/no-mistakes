import { readFileSync } from 'node:fs';

export function loadSkippedFixture() {
  return readFileSync('fixtures/schema.sql');
}
