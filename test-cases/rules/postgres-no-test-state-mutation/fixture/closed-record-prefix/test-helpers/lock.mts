import { beginTransaction } from 'db';
import { assertSafeSqlIdentifier } from './assert.mts';

export async function lockReferences(referenceTables: string[]) {
  const orderedReferences = [
    ...referenceTables.filter((t) => t === 'posts'),
    ...referenceTables.filter((t) => t !== 'posts'),
  ];
  const query = await beginTransaction();
  for (const table of orderedReferences) {
    await query(
      `/* lockReference */ LOCK TABLE ${assertSafeSqlIdentifier(table)} IN ACCESS EXCLUSIVE MODE`,
    );
  }
}
