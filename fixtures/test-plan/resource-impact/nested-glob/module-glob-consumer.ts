import { globSync } from 'glob';
import { fileURLToPath } from 'node:url';

export const moduleFixtures = globSync(
  fileURLToPath(new URL('./fixtures/*.txt', import.meta.url)),
);
