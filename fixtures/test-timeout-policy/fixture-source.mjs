import { readFileSync } from 'node:fs';

/** Saved child sources keep registration probes fixture-backed. */
export function fixtureSource(name) {
  return readFileSync(new URL(`./sources/${name}`, import.meta.url), 'utf8');
}

/** Interpolate only explicitly supplied settings into checked-in SDK sources. */
export function fixtureTemplate(name, settings = []) {
  return fixtureSource(name).replace(/\{\{(\d+)\}\}/g, (_, index) => {
    if (!(index in settings)) throw new Error(`Missing fixture setting ${index} for ${name}`);
    return String(settings[index]);
  });
}
