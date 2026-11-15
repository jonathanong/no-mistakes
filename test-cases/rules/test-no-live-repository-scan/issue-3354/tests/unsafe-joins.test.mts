import { mkdtempSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { it } from 'vitest';

declare const unknownChild: string;
declare const unknownChildren: readonly string[];

it('reports joins without a bounded child proof', () => {
  const root = mkdtempSync(join(tmpdir(), 'case-'));
  readdirSync(join(root, unknownChild));
  for (const name of ['a', '..'] as const) {
    readdirSync(join(root, name));
  }
  for (const absoluteName of ['a', '/outside'] as const) {
    readdirSync(join(root, absoluteName));
  }
  let mutableChildren = ['a', 'b'];
  mutableChildren = ['..'];
  for (const mutableName of mutableChildren) {
    readdirSync(join(root, mutableName));
  }
  for (const mixedName of ['a', unknownChild] as const) {
    readdirSync(join(root, mixedName));
  }
  for (let reassignedName of ['a', 'b']) {
    reassignedName = unknownChild;
    readdirSync(join(root, reassignedName));
  }
  for (const spreadName of ['a', ...unknownChildren]) {
    readdirSync(join(root, spreadName));
  }
  for (const windowsName of ['a', 'C:/outside'] as const) {
    readdirSync(join(root, windowsName));
  }
  for (const uncName of ['a', '//server/share'] as const) {
    readdirSync(join(root, uncName));
  }
  for (const longName of ['a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/a/../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../../..'] as const) {
    readdirSync(join(root, longName));
  }
  for (const emptyName of [] as const) {
    readdirSync(join(root, emptyName));
  }
  for (const overLimitName of ['child-1', 'child-2', 'child-3', 'child-4', 'child-5', 'child-6', 'child-7', 'child-8', 'child-9', 'child-10', 'child-11', 'child-12', 'child-13', 'child-14', 'child-15', 'child-16', 'child-17'] as const) {
    readdirSync(join(root, overLimitName));
  }
  for (const unknownName of unknownChildren) {
    readdirSync(join(root, unknownName));
  }
});
