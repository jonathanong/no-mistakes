declare function query(id: string): Promise<void>;

export async function backfillPage(ids: string[]) {
  for (const id of ids) await query(id);
}

export async function processItems(ids: string[]) {
  for (const id of ids) await query(id);
}
