import { type Query } from 'db';

function randomUUID() {
  return process.env.RAW_UUID!;
}

function createFakeTempTableName(prefix: string): string {
  return `${prefix}_${randomUUID().replaceAll('-', '')}`;
}

export async function pruneFake(q: Query) {
  const table = createFakeTempTableName('temp_recipients');
  await q(`/* pruneFake */ DELETE FROM items i USING ${table} r WHERE r.id = i.id`);
}
