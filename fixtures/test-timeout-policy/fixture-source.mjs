import { readFileSync } from 'node:fs';

/** Saved child sources keep registration probes fixture-backed. */
export function fixtureSource(name) {
  return readFileSync(new URL(`./sources/${name}`, import.meta.url), 'utf8');
}
