import { type Query } from 'db';
import { randomUUID } from 'node:crypto';
import { createTempTableName } from './temp-names.ts';

function mixedName(prefix: string, flag: boolean) {
  if (flag) return `${prefix}_${randomUUID().replaceAll('-', '')}`;
  return process.env.RAW_TABLE!;
}

function invalidatedName(prefix: string) {
  prefix = process.env.RAW_TABLE!;
  return `${prefix}_${randomUUID().replaceAll('-', '')}`;
}

function recursiveName(prefix: string): string {
  return recursiveName(prefix);
}

async function asyncName(prefix: string) {
  return createTempTableName(prefix);
}

export async function negatives(q: Query, flag: boolean) {
  const mixed = mixedName('temp_recipients', flag);
  await q(`/* mixed */ DELETE FROM items USING ${mixed}`);
  const invalidated = invalidatedName('temp_recipients');
  await q(`/* invalidated */ DELETE FROM items USING ${invalidated}`);
  const recursive = recursiveName('temp_recipients');
  await q(`/* recursive */ DELETE FROM items USING ${recursive}`);
  const unawaited = asyncName('temp_recipients');
  await q(`/* unawaited */ DELETE FROM items USING ${unawaited}`);
}
