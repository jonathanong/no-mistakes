import { query, write, beginTransaction } from '@example/db';
async function member(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function shorthand(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function directShorthand(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const query = tx.query;
  const options = { query };
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unwritten(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await query('UPDATE other_items SET active = true WHERE id = $1', [id], options);
}
async function mutated(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  options.query = query;
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function escaped(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  change(options);
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function mutable(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  let options = { query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function committed(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await tx.commit();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function committedObjectAlias(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  const options = { query: alias.query };
  await alias.commit();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function rolledBackObjectAliasChain(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  const other = alias;
  const options = { query: other.query };
  await other.rollback();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function optionsBeforeAliasCommit(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  const alias = tx;
  await alias.commit();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function optionsBeforeAliasRollbackChain(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  const alias = tx;
  const other = alias;
  await other.rollback();
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function callableAliasCommit(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  await execute('COMMIT');
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function mutatedObjectAlias(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  alias.query = query;
  const options = { query: alias.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function mutatedReceiverBeforeCapture(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  tx.query = query;
  const options = { query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function getterOptions(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { get query() { return tx.query; } };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function spreadOptions(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { ...{}, query: tx.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function nestedFunctionOptions(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  async function write() {
    await query('UPDATE items SET active = true WHERE id = $1', [id], options);
  }
  await write();
}

async function configuredWrite(id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  await write('UPDATE items SET active = true WHERE id = $1', [id], options);
}
