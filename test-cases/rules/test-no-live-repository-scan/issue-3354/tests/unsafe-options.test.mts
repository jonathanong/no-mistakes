import { mkdtempSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';
import { scanOverridden } from '../src/overridden-options.mts';
import { scanOpaque } from '../src/opaque-options.mts';

declare const baseOptions: { root?: string };
declare function opaqueOptions(root: string): { root: string };

it('reports a later opaque spread that can replace the root', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  const options = { root, ...baseOptions };
  scanOverridden(options);
});

it('reports options returned by an unproven helper', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  scanOpaque(opaqueOptions(root));
});

it('reports direct properties overwritten by spreads or assignments', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  const unsafeDirect = { root, ...baseOptions };
  readdirSync(unsafeDirect.root);
  const mutatedDirect = { root };
  mutatedDirect.root = process.cwd();
  readdirSync(mutatedDirect.root);
});
