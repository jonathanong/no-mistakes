declare function read(...args: any[]): Promise<any>;
export async function bounded(id: string) {
  const first = await read(id);
  const second = await read(first.id);
  await read(second.id);
}
export async function control(ids: string[]) { for (const id of ids) await read(id); }
