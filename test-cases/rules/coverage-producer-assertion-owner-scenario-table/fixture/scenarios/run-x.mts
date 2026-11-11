import assert from 'node:assert/strict';
async function listPage() { return []; }
export async function runX() {
  const rows = await listPage();
  assert.equal(rows.length, 0);
  assert.deepEqual(await listPage(), []);
}
export async function withoutAssertion() { await listPage(); }
