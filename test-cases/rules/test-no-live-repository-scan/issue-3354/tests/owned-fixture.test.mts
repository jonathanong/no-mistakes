import { readdirSync } from 'node:fs';
import { it } from 'vitest';

it('keeps fixture roots with string encoding options', () => {
  readdirSync('fixtures', 'utf8');
  const encoding = 'utf8';
  readdirSync('fixtures', encoding);
  const options = { withFileTypes: false };
  options.withFileTypes = true;
  readdirSync('fixtures', options);
});
