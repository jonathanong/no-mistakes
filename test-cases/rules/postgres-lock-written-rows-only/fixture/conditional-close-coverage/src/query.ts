import { query, beginTransaction } from '@example/db';
async function conditionalClose(skip: boolean, close: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    if (close) await tx.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function bothArmsClose(skip: boolean, commit: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    if (commit) await tx.commit();
    else await tx.rollback();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function unrelatedClose(skip: boolean, id: string) {
  await using tx = await beginTransaction();
  await using unrelated = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  if (skip) {
    await unrelated.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function reassignedThenReturn(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    tx = null;
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unrelatedShadowAssignment(skip: boolean, id: string) {
  await using tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  if (skip) {
    {
      let tx = 1;
      tx = 2;
    }
    await alias.commit();
    return;
  }
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function uncalledAssignment(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    function change() { tx = null; }
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function uncalledCommit(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    async function finish() { await tx.commit(); }
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function logicalClose(skip: boolean, close: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    close && await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function caughtClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    try { await tx.commit(); } catch {}
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function loopClose(skip: boolean, count: number, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    for (let index = 0; index < count; index++) await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function ternaryClose(skip: boolean, close: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    close ? await tx.commit() : null;
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function switchClose(skip: boolean, action: string, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    switch (action) { case 'commit': await tx.commit(); }
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function switchLabelClose(skip: boolean, action: string, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    switch (action) {
      case 'first': break;
      case await tx.commit(): break;
    }
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function callableMethodClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  execute.commit = async () => {};
  if (skip) {
    await execute.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function overwrittenMethodClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  tx.commit = async () => {};
  if (skip) {
    await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function overwrittenComputedClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  tx['rollback'] = async () => {};
  if (skip) {
    await tx.rollback();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function unrelatedPropertyClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  tx['label'] = 'stable';
  if (skip) {
    await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
