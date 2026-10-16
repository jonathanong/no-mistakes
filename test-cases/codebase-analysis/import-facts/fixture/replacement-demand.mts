import { read } from '@vendor/replacement-demand';
// Imports and scopes still walk even when replacement execution is not requested.
export async function run(flag) {
  for (let index = 0; index < 2; index++) {
    read();
    if (flag) continue;
  }
  for (const item of [1, 2]) { read(item); }
  while (flag) { read(); break; }
  do { read(); } while (false);
  const box = { read };
  delete box.read;
  await import('./lazy.mts');
  return read;
}
