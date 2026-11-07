import { query, beginTransaction, type TransactionQuery } from '@example/db';
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
    tx = null; // Unsupported rebinding must not prove closure; see docs/rules/postgres-lock-written-rows-only.md.
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
async function callableMemberOptions(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute.query };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function callableMemberAlias(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const nested = execute.query;
  const options = { query: nested };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function inlineCallableMemberOptions(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  await query('UPDATE items SET active = true WHERE id = $1', [id], { query: execute.query });
}
async function objectAliasMemberOptions(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const alias = tx;
  const execute = alias.query;
  const options = { query: execute };
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function sqlCommitReturn(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    await tx.query('COMMIT');
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function nestedSqlCommitReturn(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    await tx.query('SELECT 1', [await query('COMMIT')]);
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function overriddenSqlCommitReturn(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const execute = tx.query;
  const options = { query: execute };
  if (skip) {
    await execute('COMMIT', [], { query });
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function escapedReceiverClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    Object.assign(tx, { commit: async () => {} });
    await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function explicitQueryMutationBeforeHelper(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  tx.query = query;
  await knownWriteHelper(tx, id);
}
async function opaqueQueryMutationBeforeHelper(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  Object.assign(tx, { query });
  await knownWriteHelper(tx, id);
}
async function knownHelperWithUnmutatedReceiver(id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  await knownWriteHelper(tx, id);
}
async function knownWriteHelper(tx: TransactionQuery, id: string) {
  await tx.query('UPDATE items SET active = true WHERE id = $1', [id]);
}
async function optionalMethodClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    await tx.commit?.();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function optionalReceiverClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    await tx?.rollback();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
async function directMethodClose(skip: boolean, id: string) {
  const tx = await beginTransaction();
  await tx.query('SELECT id FROM items WHERE id = $1 FOR UPDATE', [id]);
  const options = { query: tx.query };
  if (skip) {
    await tx.commit();
    return;
  }
  await query('UPDATE items SET active = true WHERE id = $1', [id], options);
}
